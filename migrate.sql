ATTACH "../../../.hyperion/db/hyperion.db" AS db1;
ATTACH "hyperion.db" AS db2;

BEGIN TRANSACTION;
	INSERT INTO db2.instances SELECT * FROM db1.instances;
	INSERT INTO db2.auth SELECT * FROM db1.auth;
	INSERT INTO db2.meta SELECT * FROM db1.meta;
	INSERT INTO db2.settings SELECT * FROM db1.settings;
COMMIT;
