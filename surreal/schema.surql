OPTION IMPORT;

DEFINE ACCESS account ON DATABASE TYPE RECORD
  SIGNUP (
    CREATE user SET
      email    = $email,
      username = $username,
      password = crypto::argon2::generate($password),
      created  = time::now()
  )
  SIGNIN (
    SELECT * FROM user
    WHERE email = $email
      AND crypto::argon2::compare(password, $password)
  );

DEFINE TABLE user SCHEMAFULL
  PERMISSIONS
    FOR select WHERE id = $auth OR $auth IN (SELECT owner FROM contact WHERE target = id)
    FOR update WHERE id = $auth
    FOR create NONE
    FOR delete NONE;
DEFINE FIELD username  ON user TYPE string;
DEFINE FIELD email     ON user TYPE string;
DEFINE FIELD password  ON user TYPE string;
DEFINE FIELD avatar    ON user TYPE option<string>;
DEFINE FIELD created   ON user TYPE datetime DEFAULT time::now();
DEFINE INDEX email_idx ON user FIELDS email UNIQUE;

DEFINE TABLE room SCHEMAFULL
  PERMISSIONS
    FOR select, create FULL
    FOR update, delete NONE;
DEFINE FIELD name    ON room TYPE string;
DEFINE FIELD created ON room TYPE datetime DEFAULT time::now();

DEFINE TABLE message SCHEMAFULL
  PERMISSIONS
    FOR select FULL
    FOR create WHERE author = $auth
    FOR update WHERE author = $auth
    FOR delete WHERE author = $auth;
DEFINE FIELD room            ON message TYPE record<room>;
DEFINE FIELD author          ON message TYPE record<user>;
DEFINE FIELD author_username ON message TYPE option<string>;
DEFINE FIELD body            ON message TYPE string;
DEFINE FIELD created         ON message TYPE datetime DEFAULT time::now();

DEFINE TABLE contact SCHEMAFULL
  PERMISSIONS
    FOR select WHERE owner = $auth
    FOR create WHERE owner = $auth
    FOR delete WHERE owner = $auth
    FOR update NONE;
DEFINE FIELD owner  ON contact TYPE record<user>;
DEFINE FIELD target ON contact TYPE record<user>;
DEFINE INDEX unique_contact ON contact FIELDS owner, target UNIQUE;
