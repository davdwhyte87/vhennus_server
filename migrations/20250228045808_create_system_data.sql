-- Add migration script here
CREATE TABLE system_data(
    id INT PRIMARY KEY,
    price DECIMAL(24, 3) NOT NULL ,
    android_app_version VARCHAR NOT NULL,
    trivia_win_amount DECIMAL(24,3) NOT NULL
)