-- Add up migration script here
DROP TABLE IF EXISTS orders;
DROP TABLE IF EXISTS regions;
CREATE TABLE orders (
    id INT PRIMARY KEY,
    region_id INT,
    gift_name VARCHAR(50),
    quantity INT
);
CREATE TABLE regions (id INT PRIMARY KEY, name VARCHAR(50))