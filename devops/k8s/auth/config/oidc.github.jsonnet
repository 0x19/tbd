// Maps GitHub's user + e-mail claims (scope user:email) to identity traits.
local claims = std.extVar('claims');
{
  identity: {
    traits: {
      [if 'email' in claims && claims.email != null then 'email' else null]: claims.email,
      [if 'name' in claims && claims.name != null then 'name' else null]: claims.name,
    },
  },
}
