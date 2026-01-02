//! PIM role cache with time-to-live (TTL) support.

use chrono::{DateTime, Duration, Utc};

use super::models::EligibleRole;

/// Cache TTL in hours.
const CACHE_TTL_HOURS: i64 = 1;

/// Cached data with timestamp.
#[derive(Debug, Clone)]
struct CachedData<T> {
    data: T,
    cached_at: DateTime<Utc>,
}

/// PIM role cache with 1-hour TTL.
#[derive(Debug, Clone, Default)]
pub struct PimCache {
    /// Cached eligible roles.
    eligible_roles: Option<CachedData<Vec<EligibleRole>>>,
}

impl PimCache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get cached eligible roles if still valid.
    pub fn get_eligible_roles(&self) -> Option<&Vec<EligibleRole>> {
        self.eligible_roles.as_ref().and_then(|cached| {
            if self.is_valid(&cached.cached_at) {
                Some(&cached.data)
            } else {
                None
            }
        })
    }

    /// Store eligible roles in cache.
    pub fn set_eligible_roles(&mut self, roles: Vec<EligibleRole>) {
        self.eligible_roles = Some(CachedData {
            data: roles,
            cached_at: Utc::now(),
        });
    }

    /// Get when the cache was last updated.
    pub fn cached_at(&self) -> Option<DateTime<Utc>> {
        self.eligible_roles.as_ref().map(|c| c.cached_at)
    }

    /// Invalidate the cache.
    pub fn invalidate(&mut self) {
        self.eligible_roles = None;
    }

    /// Check if the cache is still valid.
    pub fn is_valid(&self, cached_at: &DateTime<Utc>) -> bool {
        Utc::now() - *cached_at < Duration::hours(CACHE_TTL_HOURS)
    }

    /// Check if cache needs refresh.
    pub fn needs_refresh(&self) -> bool {
        match &self.eligible_roles {
            Some(cached) => !self.is_valid(&cached.cached_at),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_role() -> EligibleRole {
        EligibleRole {
            id: "test-id".to_string(),
            role_definition_id: "role-def".to_string(),
            role_name: "Contributor".to_string(),
            subscription_id: "sub-id".to_string(),
            subscription_name: "Test Sub".to_string(),
            scope: "/subscriptions/sub-id".to_string(),
            principal_id: "principal".to_string(),
        }
    }

    #[test]
    fn test_cache_empty() {
        let cache = PimCache::new();
        assert!(cache.get_eligible_roles().is_none());
        assert!(cache.needs_refresh());
    }

    #[test]
    fn test_cache_set_get() {
        let mut cache = PimCache::new();
        let roles = vec![make_test_role()];

        cache.set_eligible_roles(roles.clone());

        let cached = cache.get_eligible_roles();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);
        assert!(!cache.needs_refresh());
    }

    #[test]
    fn test_cache_invalidate() {
        let mut cache = PimCache::new();
        cache.set_eligible_roles(vec![make_test_role()]);

        cache.invalidate();

        assert!(cache.get_eligible_roles().is_none());
        assert!(cache.needs_refresh());
    }
}
