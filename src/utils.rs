use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use rustls::{Certificate, PrivateKey};
use tokio::fs;

pub async fn new_tls_key_pair(cert_path: &PathBuf, key_path: &PathBuf) -> anyhow::Result<(Vec<Certificate>, PrivateKey)> {
    let key_file = fs::read(key_path).await.context("failed to read private key")?;

    // Try to parse the key as an RSA key, then as a PKCS8 key, then as an ECC key
    let keys = {
        if let Ok(rsa_private_keys) = rustls_pemfile::rsa_private_keys(&mut &*key_file) {
            Some(rsa_private_keys)
        } else if let Ok(pkcs8_private_keys) = rustls_pemfile::pkcs8_private_keys(&mut &*key_file) {
            Some(pkcs8_private_keys)
        } else if let Ok(ecc_private_keys) = rustls_pemfile::ec_private_keys(&mut &*key_file) {
            Some(ecc_private_keys)
        } else {
            return Err(anyhow::anyhow!("failed to parse private key"));
        }
    };

    let key = PrivateKey(keys.unwrap().remove(0));

    let cert_file = fs::read(cert_path).await.context("failed to read certificate chain")?;
    let cert_chain = rustls_pemfile::certs(&mut &*cert_file)?
        .into_iter()
        .map(|data| Certificate(data))
        .collect::<Vec<_>>();

    Ok((cert_chain, key))
}

fn kmp_table<T: PartialEq>(pattern: &[T]) -> Vec<usize> {
    let mut prefix = vec![0; pattern.len()];
    let mut j = 0;

    for i in 1..pattern.len() {
        while j > 0 && pattern[i] != pattern[j] {
            j = prefix[j - 1];
        }

        if pattern[i] == pattern[j] {
            j += 1;
            prefix[i] = j;
        }
    }

    prefix
}

fn kmp_search<T: PartialEq>(haystack: &[T], needle: &[T]) -> bool {
    if needle.is_empty() {
        return true;
    }

    let prefix_table = kmp_table(needle);
    let mut j = 0;

    for i in 0..haystack.len() {
        while j > 0 && haystack[i] != needle[j] {
            j = prefix_table[j - 1];
        }

        if haystack[i] == needle[j] {
            if j == needle.len() - 1 {
                return true;
            }
            j += 1;
        }
    }

    false
}

pub type Matcher = Arc<dyn Fn(String) -> bool + Send + Sync + 'static>;

/// Create a domain suffix matcher
fn domain_suffix_match_fn(domain: &str) -> Matcher {
    let domain_parts = domain
        .trim()
        .split('.')
        .rev()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    Arc::new(move |test_domain: String| {
        let parts = test_domain.split('.').rev().filter(|s| !s.is_empty()).collect::<Vec<_>>();

        if domain_parts.len() > parts.len() {
            return false;
        }

        for (a, b) in domain_parts.iter().zip(parts.iter()) {
            if a != b {
                return false;
            }
        }

        true
    })
}

fn domain_full_match_fn(domain: &str) -> Matcher {
    let domain_parts = domain
        .trim()
        .split('.')
        .rev()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    Arc::new(move |test_domain: String| {
        let parts = test_domain.split('.').rev().filter(|s| !s.is_empty()).collect::<Vec<_>>();

        if domain_parts.len() != parts.len() {
            return false;
        }

        for (a, b) in domain_parts.iter().zip(parts.iter()) {
            if a != b {
                return false;
            }
        }

        true
    })
}

fn domain_keyword_match_fn(domain: &str) -> Matcher {
    let domain_parts = domain
        .trim()
        .split('.')
        .rev()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    Arc::new(move |test_domain: String| {
        let parts = test_domain
            .trim()
            .split('.')
            .rev()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        return kmp_search(&parts, &domain_parts);
    })
}

fn domain_regexp_match_fn(rule: &str) -> anyhow::Result<Matcher> {
    let re = regex::Regex::new(rule)?;
    Ok(Arc::new(move |test_domain: String| re.is_match(&test_domain)))
}

pub fn parse_domain(domain: &str) -> anyhow::Result<Matcher> {
    let domain = domain.trim();
    if domain.is_empty() {
        return Err(anyhow::anyhow!("domain is empty"));
    }

    if domain.find(':').is_none() {
        return Ok(domain_suffix_match_fn(domain));
    }

    if domain.starts_with("domain:") {
        let domain = domain.trim_start_matches("domain:");
        return Ok(domain_suffix_match_fn(domain));
    }

    if domain.starts_with("full:") {
        let domain = domain.trim_start_matches("full:");
        return Ok(domain_full_match_fn(domain));
    }

    if domain.starts_with("keyword:") {
        let keyword = domain.trim_start_matches("keyword:");
        return Ok(domain_keyword_match_fn(keyword));
    }

    if domain.starts_with("regexp:") {
        let regex = domain.trim_start_matches("regexp:");
        return Ok(domain_regexp_match_fn(regex)?);
    }

    return Err(anyhow::anyhow!("domain is invalid"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kmp_search() {
        let test_cases = vec![
            (vec![1, 2, 3, 4, 5], vec![2, 3, 4], true),
            (vec![1, 2, 3, 4, 5], vec![2, 4, 3], false),
            (vec![1, 2, 3], vec![], true),
            (vec![], vec![1, 2, 3], false),
            (vec![1, 2, 3], vec![1, 2, 3], true),
            (vec![1, 2, 3], vec![1, 2, 3, 4], false),
            (vec![1, 2, 2, 3, 2], vec![2, 2, 3], true),
            (vec![1, 2, 3, 2, 3, 4], vec![2, 3], true),
        ];

        for (i, (haystack, needle, expected)) in test_cases.iter().enumerate() {
            assert_eq!(kmp_search(haystack, needle), *expected, "Test case {} failed", i + 1);
        }
    }

    #[test]
    fn test_domain_suffix_match_fn() {
        let matcher = domain_suffix_match_fn("example.com");

        assert!(matcher("example.com".to_string()));
        assert!(matcher("www.example.com".to_string()));
        assert!(matcher("sub.www.example.com".to_string()));

        assert!(!matcher("example.org".to_string()));
        assert!(!matcher("example.com.org".to_string()));
        assert!(!matcher("example".to_string()));
    }

    #[test]
    fn test_domain_full_match_fn() {
        let matcher = domain_full_match_fn("example.com");

        assert!(matcher("example.com".to_string()));

        assert!(!matcher("www.example.com".to_string()));
        assert!(!matcher("sub.www.example.com".to_string()));
        assert!(!matcher("example.org".to_string()));
        assert!(!matcher("example.com.org".to_string()));
        assert!(!matcher("example".to_string()));
    }

    #[test]
    fn test_domain_keyword_match_fn() {
        let matcher = domain_keyword_match_fn("example.com");

        assert!(matcher("example.com".to_string()));
        assert!(matcher("www.example.com".to_string()));
        assert!(matcher("sub.www.example.com".to_string()));
        assert!(matcher("example.com.org".to_string()));
        assert!(matcher("www.example.com.org".to_string()));

        assert!(!matcher("example.org".to_string()));
        assert!(!matcher("example".to_string()));
        assert!(!matcher("example.foobar.com".to_string()));
    }

    #[test]
    fn test_parse_domain() {
        let matcher = parse_domain("domain:google.com").unwrap();
        assert!(matcher("google.com".to_string()));
        assert!(matcher("www.google.com".to_string()));
        assert!(!matcher("www.google.com.cn".to_string()));
        assert!(!matcher("example.com".to_string()));

        let matcher = parse_domain("full:google.com").unwrap();
        assert!(matcher("google.com".to_string()));
        assert!(!matcher("www.google.com".to_string()));
        assert!(!matcher("www.google.com.cn".to_string()));

        let matcher = parse_domain("keyword:google.com").unwrap();
        assert!(matcher("google.com".to_string()));
        assert!(matcher("www.google.com".to_string()));
        assert!(matcher("maps.l.google.com".to_string()));
        assert!(matcher("www.google.com.cn".to_string()));
        assert!(!matcher("example.com".to_string()));

        let matcher = parse_domain("regexp:^google\\.*").unwrap();
        assert!(matcher("google.com".to_string()));
        assert!(matcher("google.cn".to_string()));
        assert!(!matcher("www.google.com".to_string()));
    }
}
