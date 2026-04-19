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
    FOR select WHERE $auth != NONE
    FOR update WHERE id = $auth
    FOR create NONE
    FOR delete NONE;
DEFINE FIELD username  ON user TYPE string;
DEFINE FIELD email     ON user TYPE string;
DEFINE FIELD password  ON user TYPE string;
DEFINE FIELD avatar    ON user TYPE option<string>;
DEFINE FIELD created   ON user TYPE datetime DEFAULT time::now();
DEFINE INDEX email_idx ON user FIELDS email UNIQUE;
DEFINE INDEX username_idx ON user FIELDS username;

DEFINE TABLE room SCHEMAFULL
  PERMISSIONS
    FOR select WHERE created_by = $auth OR id IN (SELECT VALUE room FROM room_member WHERE user = $auth) OR kind = "public"
    FOR create FULL
    FOR update WHERE id IN (SELECT VALUE room FROM room_member WHERE user = $auth AND role = "owner")
    FOR delete NONE;
DEFINE FIELD name       ON room TYPE option<string>;
DEFINE FIELD kind       ON room TYPE string DEFAULT "public" ASSERT $value IN ["public", "private", "direct"];
DEFINE FIELD created_by ON room TYPE option<record<user>>;
DEFINE FIELD direct_key ON room TYPE option<string>;
DEFINE FIELD created ON room TYPE datetime DEFAULT time::now();
DEFINE FIELD updated ON room TYPE datetime DEFAULT time::now();
DEFINE INDEX direct_key_idx ON room FIELDS direct_key UNIQUE;

DEFINE TABLE room_member SCHEMAFULL
  PERMISSIONS
    FOR select WHERE room IN (SELECT VALUE room FROM room_member WHERE user = $auth)
    FOR create WHERE user = $auth OR room IN (SELECT VALUE room FROM room_member WHERE user = $auth AND role = "owner")
    FOR update WHERE user = $auth OR room IN (SELECT VALUE room FROM room_member WHERE user = $auth AND role = "owner")
    FOR delete WHERE user = $auth OR room IN (SELECT VALUE room FROM room_member WHERE user = $auth AND role = "owner");
DEFINE FIELD room         ON room_member TYPE record<room>;
DEFINE FIELD user         ON room_member TYPE record<user>;
DEFINE FIELD role         ON room_member TYPE string DEFAULT "member" ASSERT $value IN ["owner", "member"];
DEFINE FIELD joined       ON room_member TYPE datetime DEFAULT time::now();
DEFINE FIELD last_read_at ON room_member TYPE option<datetime>;
DEFINE FIELD muted        ON room_member TYPE bool DEFAULT false;
DEFINE INDEX unique_room_member ON room_member FIELDS room, user UNIQUE;

DEFINE TABLE message SCHEMAFULL
  PERMISSIONS
    FOR select WHERE room.kind = "public" OR room IN (SELECT VALUE room FROM room_member WHERE user = $auth)
    FOR create WHERE author = $auth AND (room.kind = "public" OR room IN (SELECT VALUE room FROM room_member WHERE user = $auth))
    FOR update WHERE author = $auth
    FOR delete WHERE author = $auth;
DEFINE FIELD room            ON message TYPE record<room>;
DEFINE FIELD author          ON message TYPE record<user>;
DEFINE FIELD author_username ON message TYPE option<string>;
DEFINE FIELD body            ON message TYPE string ASSERT string::len($value) <= 4000;
DEFINE FIELD created         ON message TYPE datetime DEFAULT time::now();
DEFINE FIELD updated         ON message TYPE option<datetime>;
DEFINE FIELD deleted         ON message TYPE bool DEFAULT false;
DEFINE FIELD reply_to        ON message TYPE option<record<message>>;
DEFINE INDEX message_room_created ON message FIELDS room, created;

DEFINE TABLE message_reaction SCHEMAFULL
  PERMISSIONS
    FOR select WHERE message.room.kind = "public" OR message.room IN (SELECT VALUE room FROM room_member WHERE user = $auth)
    FOR create WHERE user = $auth AND (message.room.kind = "public" OR message.room IN (SELECT VALUE room FROM room_member WHERE user = $auth))
    FOR update NONE
    FOR delete WHERE user = $auth;
DEFINE FIELD message ON message_reaction TYPE record<message>;
DEFINE FIELD user    ON message_reaction TYPE record<user>;
DEFINE FIELD emoji   ON message_reaction TYPE string ASSERT string::len($value) <= 16;
DEFINE FIELD created ON message_reaction TYPE datetime DEFAULT time::now();
DEFINE INDEX unique_message_reaction ON message_reaction FIELDS message, user, emoji UNIQUE;

DEFINE TABLE contact SCHEMAFULL
  PERMISSIONS
    FOR select WHERE owner = $auth
    FOR create WHERE owner = $auth
    FOR delete WHERE owner = $auth
    FOR update NONE;
DEFINE FIELD owner  ON contact TYPE record<user>;
DEFINE FIELD target ON contact TYPE record<user>;
DEFINE INDEX unique_contact ON contact FIELDS owner, target UNIQUE;
