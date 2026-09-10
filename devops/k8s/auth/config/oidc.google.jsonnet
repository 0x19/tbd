// Maps Google's ID token claims to identity traits. Only a verified e-mail is
// accepted as the identifier; Google's assertion also marks the address verified
// in Kratos, so no verification e-mail is needed for Google sign-ups.
local claims = { email_verified: false } + std.extVar('claims');
{
  identity: {
    traits: {
      [if 'email' in claims && claims.email_verified then 'email' else null]: claims.email,
      [if 'name' in claims then 'name' else null]: claims.name,
    },
    [if 'email' in claims && claims.email_verified then 'verified_addresses' else null]: [
      { via: 'email', value: claims.email },
    ],
  },
}
