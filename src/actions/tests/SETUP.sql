do $$
begin

TRUNCATE guilds, colours CASCADE;

INSERT INTO guilds VALUES 
    (482110165651554322, 482110165651554327, '{}'::json);

INSERT INTO colours VALUES
    (483501321945612319, 'Red', 482110165651554322);

INSERT INTO colours VALUES 
    (483501363708297225, 'Green', 482110165651554322);

end;
$$