-- create regions table
CREATE TABLE regions (id INT PRIMARY KEY, name VARCHAR(50));
-- create orders table
CREATE TABLE orders (
    id INT PRIMARY KEY,
    region_id INT,
    gift_name VARCHAR(50),
    quantity INT
);