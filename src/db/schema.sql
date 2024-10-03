CREATE TABLE IF NOT EXISTS category (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    points TEXT[]
);

CREATE TABLE IF NOT EXISTS call (
    id UUID PRIMARY KEY NOT NULL,  
    name VARCHAR(255),
    location VARCHAR(255),
    emotional_tone VARCHAR(50),
    text TEXT NOT NULL,
    categories TEXT[]
);
INSERT INTO category (title, points)
VALUES 
    ('Visa and Passport Services', ARRAY['Border crossing', 'International documentation']),
    ('Diplomatic Inquiries', ARRAY['Embassy services', 'Foreign relations']),
    ('Travel Advisories', ARRAY['Travel restrictions', 'Health and safety guidelines']),
    ('Consular Assistance', ARRAY['Emergency assistance', 'Legal aid']),
    ('Trade and Economic Cooperation', ARRAY['Bilateral trade', 'Investment opportunities']);


