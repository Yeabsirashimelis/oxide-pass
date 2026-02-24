ALTER TABLE apps
    ADD COLUMN working_dir TEXT NOT NULL DEFAULT '/tmp';
