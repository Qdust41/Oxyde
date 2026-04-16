-- surreal/auth.surql
-- Run after schema.surql.
-- SURREAL_JWT_SECRET must be set as an env var when starting the SurrealDB process.
-- The key is read at runtime via env::get() — nothing needs to be changed in this file.

DEFINE ACCESS account ON DATABASE TYPE RECORD
  SIGNUP (
    CREATE user SET
      email    = $email,
      username = $username,
      password = crypto::argon2::generate($password)
  )
  SIGNIN (
    SELECT * FROM user
    WHERE email = $email
    AND crypto::argon2::compare(password, $password)
  )
  WITH JWT ALGORITHM HS512 KEY env::get("SURREAL_JWT_SECRET");
