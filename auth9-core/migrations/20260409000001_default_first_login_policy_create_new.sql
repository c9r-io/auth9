-- Switch the column default for first_login_policy from 'auto_merge' to 'create_new'.
-- This only affects newly-created IdPs/connectors that do not explicitly specify a policy.
-- Existing rows are intentionally left untouched — admins who depend on 'auto_merge'
-- keep their configuration; they should review and update manually if desired.

ALTER TABLE social_providers
    ALTER COLUMN first_login_policy SET DEFAULT 'create_new';

ALTER TABLE enterprise_sso_connectors
    ALTER COLUMN first_login_policy SET DEFAULT 'create_new';
