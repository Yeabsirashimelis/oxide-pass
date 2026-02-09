ALTER TABLE apps 
    ALTER COLUMN id SET DEFAULT gen_random_uuid();