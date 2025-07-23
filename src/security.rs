use crate::error::PreviewError;
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use url::Url;

/// Configuration for URL validation
#[derive(Debug, Clone)]
pub struct UrlValidationConfig {
    /// Allowed URL schemes (default: ["http", "https"])
    pub allowed_schemes: HashSet<String>,
    /// Block private/local IP addresses (default: true)
    pub block_private_ips: bool,
    /// Block localhost addresses (default: true)
    pub block_localhost: bool,
    /// Domain blacklist
    pub blocked_domains: HashSet<String>,
    /// Domain whitelist (if not empty, only these domains are allowed)
    pub allowed_domains: HashSet<String>,
    /// Maximum number of redirects allowed
    pub max_redirects: usize,
}

impl Default for UrlValidationConfig {
    fn default() -> Self {
        let mut allowed_schemes = HashSet::new();
        allowed_schemes.insert("http".to_string());
        allowed_schemes.insert("https".to_string());

        Self {
            allowed_schemes,
            block_private_ips: true,
            block_localhost: true,
            blocked_domains: HashSet::new(),
            allowed_domains: HashSet::new(),
            max_redirects: 10,
        }
    }
}

/// Validates a URL according to security policies
#[derive(Clone)]
pub struct UrlValidator {
    config: UrlValidationConfig,
}

impl UrlValidator {
    pub fn new(config: UrlValidationConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(UrlValidationConfig::default())
    }

    /// Validates a URL string
    pub fn validate(&self, url_str: &str) -> Result<Url, PreviewError> {
        // Parse the URL
        let url = Url::parse(url_str).map_err(PreviewError::UrlParseError)?;

        // Check scheme
        if !self.config.allowed_schemes.contains(url.scheme()) {
            return Err(PreviewError::InvalidUrlScheme(url.scheme().to_string()));
        }

        // Extract host
        let host = url
            .host_str()
            .ok_or_else(|| PreviewError::InvalidUrl("No host in URL".to_string()))?;

        // Check domain whitelist/blacklist
        if !self.config.allowed_domains.is_empty() {
            if !self.is_domain_allowed(host) {
                return Err(PreviewError::DomainNotAllowed(host.to_string()));
            }
        } else if self.is_domain_blocked(host) {
            return Err(PreviewError::DomainBlocked(host.to_string()));
        }

        // Check for localhost
        if self.config.block_localhost && self.is_localhost(host) {
            return Err(PreviewError::LocalhostBlocked);
        }

        // Check for private IPs if host is an IP address
        if self.config.block_private_ips {
            // Handle IPv6 addresses which may be wrapped in brackets
            let ip_str = if host.starts_with('[') && host.ends_with(']') {
                &host[1..host.len() - 1]
            } else {
                host
            };
            
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                if self.is_private_ip(&ip) {
                    return Err(PreviewError::PrivateIpBlocked(ip.to_string()));
                }
            }
        }

        Ok(url)
    }

    fn is_domain_allowed(&self, host: &str) -> bool {
        self.config
            .allowed_domains
            .iter()
            .any(|allowed| host == allowed || host.ends_with(&format!(".{allowed}")))
    }

    fn is_domain_blocked(&self, host: &str) -> bool {
        self.config
            .blocked_domains
            .iter()
            .any(|blocked| host == blocked || host.ends_with(&format!(".{blocked}")))
    }

    fn is_localhost(&self, host: &str) -> bool {
        matches!(host, "localhost" | "127.0.0.1" | "::1" | "[::1]")
    }

    fn is_private_ip(&self, ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                ipv4.is_private()
                    || ipv4.is_loopback()
                    || ipv4.is_link_local()
                    || ipv4.is_unspecified()
                    || self.is_ipv4_reserved(ipv4)
            }
            IpAddr::V6(ipv6) => {
                ipv6.is_loopback()
                    || ipv6.is_unspecified()
                    || self.is_ipv6_link_local(ipv6)
                    || self.is_ipv6_unique_local(ipv6)
            }
        }
    }

    fn is_ipv4_reserved(&self, ip: &Ipv4Addr) -> bool {
        // Check for additional reserved ranges
        let octets = ip.octets();

        // 0.0.0.0/8
        octets[0] == 0
            // 10.0.0.0/8
            || octets[0] == 10
            // 100.64.0.0/10 (Carrier-grade NAT)
            || (octets[0] == 100 && (octets[1] & 0b11000000) == 0b01000000)
            // 169.254.0.0/16 (Link-local)
            || (octets[0] == 169 && octets[1] == 254)
            // 172.16.0.0/12
            || (octets[0] == 172 && (octets[1] >= 16 && octets[1] <= 31))
            // 192.168.0.0/16
            || (octets[0] == 192 && octets[1] == 168)
            // 224.0.0.0/4 (Multicast)
            || (octets[0] & 0b11110000) == 0b11100000
            // 240.0.0.0/4 (Reserved)
            || (octets[0] & 0b11110000) == 0b11110000
    }

    fn is_ipv6_link_local(&self, ip: &Ipv6Addr) -> bool {
        // fe80::/10
        let segments = ip.segments();
        (segments[0] & 0xffc0) == 0xfe80
    }

    fn is_ipv6_unique_local(&self, ip: &Ipv6Addr) -> bool {
        // fc00::/7
        let segments = ip.segments();
        (segments[0] & 0xfe00) == 0xfc00
    }
}

/// Content size and time limits configuration
#[derive(Debug, Clone)]
pub struct ContentLimits {
    /// Maximum content size in bytes (default: 10MB)
    pub max_content_size: usize,
    /// Maximum download time in seconds (default: 30s)
    pub max_download_time: u64,
    /// Allowed content types (if not empty, only these are allowed)
    pub allowed_content_types: HashSet<String>,
}

impl Default for ContentLimits {
    fn default() -> Self {
        let mut allowed_types = HashSet::new();
        allowed_types.insert("text/html".to_string());
        allowed_types.insert("application/xhtml+xml".to_string());
        allowed_types.insert("text/plain".to_string());
        allowed_types.insert("application/json".to_string());

        Self {
            max_content_size: 10 * 1024 * 1024, // 10MB
            max_download_time: 30,
            allowed_content_types: allowed_types,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_validator_schemes() {
        let validator = UrlValidator::with_default_config();

        assert!(validator.validate("https://example.com").is_ok());
        assert!(validator.validate("http://example.com").is_ok());
        assert!(validator.validate("ftp://example.com").is_err());
        assert!(validator.validate("file:///etc/passwd").is_err());
    }

    #[test]
    fn test_url_validator_localhost() {
        let validator = UrlValidator::with_default_config();

        assert!(validator.validate("http://localhost").is_err());
        assert!(validator.validate("http://127.0.0.1").is_err());
        assert!(validator.validate("http://[::1]").is_err());
    }

    #[test]
    fn test_url_validator_private_ips() {
        let validator = UrlValidator::with_default_config();

        assert!(validator.validate("http://10.0.0.1").is_err());
        assert!(validator.validate("http://192.168.1.1").is_err());
        assert!(validator.validate("http://172.16.0.1").is_err());
        assert!(validator.validate("http://169.254.1.1").is_err());
    }

    #[test]
    fn test_url_validator_domain_lists() {
        let mut config = UrlValidationConfig::default();
        config.blocked_domains.insert("evil.com".to_string());
        let validator = UrlValidator::new(config);

        assert!(validator.validate("http://evil.com").is_err());
        assert!(validator.validate("http://sub.evil.com").is_err());
        assert!(validator.validate("http://good.com").is_ok());
    }

    #[test]
    fn test_url_validator_whitelist() {
        let mut config = UrlValidationConfig::default();
        config.allowed_domains.insert("trusted.com".to_string());
        let validator = UrlValidator::new(config);

        assert!(validator.validate("http://trusted.com").is_ok());
        assert!(validator.validate("http://sub.trusted.com").is_ok());
        assert!(validator.validate("http://untrusted.com").is_err());
    }
}
