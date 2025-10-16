-- ============================================================================
-- SurrealDB Test Data
-- ============================================================================
-- Sample schema and data for Cleanroom Framework integration testing
--
-- This file demonstrates:
--   - Namespace and database definition
--   - Schema creation with type safety
--   - Sample data insertion
--   - Common query patterns
--   - Relationships between tables
--
-- Usage:
--   1. Via SurrealDB CLI:
--      surreal import --conn http://localhost:8000 --user root --pass root \
--        --ns test --db test test-data.sql
--
--   2. Via HTTP API:
--      curl -X POST -H "Content-Type: application/sql" \
--        -u root:root --data-binary @test-data.sql \
--        http://localhost:8000/sql
--
--   3. Via Cleanroom test step:
--      [[steps]]
--      name = "import_test_data"
--      command = ["import", "test-data.sql"]
-- ============================================================================

-- ----------------------------------------------------------------------------
-- SECTION 1: Namespace and Database Setup
-- ----------------------------------------------------------------------------

-- Define test namespace (isolated environment)
USE NS test;

-- Define test database within namespace
USE DB test;

-- ----------------------------------------------------------------------------
-- SECTION 2: Schema Definition - Users Table
-- ----------------------------------------------------------------------------

-- Create users table with full schema
DEFINE TABLE users SCHEMAFULL;

-- Define fields with type constraints
DEFINE FIELD username ON users TYPE string
    ASSERT $value != NONE AND string::len($value) >= 3;

DEFINE FIELD email ON users TYPE string
    ASSERT $value != NONE AND string::is::email($value);

DEFINE FIELD age ON users TYPE int
    ASSERT $value >= 18 AND $value <= 120;

DEFINE FIELD status ON users TYPE string
    VALUE $value OR "active"
    ASSERT $value INSIDE ["active", "inactive", "suspended"];

DEFINE FIELD created_at ON users TYPE datetime
    VALUE time::now();

DEFINE FIELD updated_at ON users TYPE datetime
    VALUE time::now();

-- Create indexes for common queries
DEFINE INDEX idx_username ON users FIELDS username UNIQUE;
DEFINE INDEX idx_email ON users FIELDS email UNIQUE;
DEFINE INDEX idx_status ON users FIELDS status;

-- ----------------------------------------------------------------------------
-- SECTION 3: Schema Definition - Products Table
-- ----------------------------------------------------------------------------

DEFINE TABLE products SCHEMAFULL;

DEFINE FIELD name ON products TYPE string
    ASSERT $value != NONE AND string::len($value) >= 2;

DEFINE FIELD description ON products TYPE string;

DEFINE FIELD price ON products TYPE decimal
    ASSERT $value > 0;

DEFINE FIELD category ON products TYPE string
    ASSERT $value INSIDE ["Electronics", "Furniture", "Clothing", "Books", "Toys"];

DEFINE FIELD in_stock ON products TYPE bool
    VALUE $value OR true;

DEFINE FIELD quantity ON products TYPE int
    VALUE $value OR 0
    ASSERT $value >= 0;

DEFINE FIELD created_at ON products TYPE datetime
    VALUE time::now();

-- Create indexes
DEFINE INDEX idx_category ON products FIELDS category;
DEFINE INDEX idx_in_stock ON products FIELDS in_stock;

-- ----------------------------------------------------------------------------
-- SECTION 4: Schema Definition - Orders Table
-- ----------------------------------------------------------------------------

DEFINE TABLE orders SCHEMAFULL;

DEFINE FIELD user_id ON orders TYPE record(users)
    ASSERT $value != NONE;

DEFINE FIELD product_id ON orders TYPE record(products)
    ASSERT $value != NONE;

DEFINE FIELD quantity ON orders TYPE int
    ASSERT $value > 0;

DEFINE FIELD total_price ON orders TYPE decimal
    ASSERT $value > 0;

DEFINE FIELD status ON orders TYPE string
    VALUE $value OR "pending"
    ASSERT $value INSIDE ["pending", "processing", "shipped", "delivered", "cancelled"];

DEFINE FIELD order_date ON orders TYPE datetime
    VALUE time::now();

DEFINE FIELD notes ON orders TYPE string;

-- Create indexes
DEFINE INDEX idx_user_orders ON orders FIELDS user_id;
DEFINE INDEX idx_product_orders ON orders FIELDS product_id;
DEFINE INDEX idx_order_status ON orders FIELDS status;
DEFINE INDEX idx_order_date ON orders FIELDS order_date;

-- ----------------------------------------------------------------------------
-- SECTION 5: Schema Definition - Reviews Table
-- ----------------------------------------------------------------------------

DEFINE TABLE reviews SCHEMAFULL;

DEFINE FIELD product_id ON reviews TYPE record(products)
    ASSERT $value != NONE;

DEFINE FIELD user_id ON reviews TYPE record(users)
    ASSERT $value != NONE;

DEFINE FIELD rating ON reviews TYPE int
    ASSERT $value >= 1 AND $value <= 5;

DEFINE FIELD comment ON reviews TYPE string;

DEFINE FIELD helpful_count ON reviews TYPE int
    VALUE $value OR 0
    ASSERT $value >= 0;

DEFINE FIELD created_at ON reviews TYPE datetime
    VALUE time::now();

-- Create indexes
DEFINE INDEX idx_product_reviews ON reviews FIELDS product_id;
DEFINE INDEX idx_user_reviews ON reviews FIELDS user_id;
DEFINE INDEX idx_rating ON reviews FIELDS rating;

-- ----------------------------------------------------------------------------
-- SECTION 6: Sample Data - Users
-- ----------------------------------------------------------------------------

-- Insert test users
CREATE users:alice SET
    username = "alice",
    email = "alice@example.com",
    age = 30,
    status = "active";

CREATE users:bob SET
    username = "bob",
    email = "bob@example.com",
    age = 25,
    status = "active";

CREATE users:charlie SET
    username = "charlie",
    email = "charlie@example.com",
    age = 35,
    status = "active";

CREATE users:david SET
    username = "david",
    email = "david@example.com",
    age = 28,
    status = "inactive";

CREATE users:eve SET
    username = "eve",
    email = "eve@example.com",
    age = 42,
    status = "active";

-- ----------------------------------------------------------------------------
-- SECTION 7: Sample Data - Products
-- ----------------------------------------------------------------------------

-- Electronics
CREATE products:laptop SET
    name = "Professional Laptop",
    description = "High-performance laptop for developers",
    price = 999.99,
    category = "Electronics",
    in_stock = true,
    quantity = 15;

CREATE products:mouse SET
    name = "Wireless Mouse",
    description = "Ergonomic wireless mouse",
    price = 29.99,
    category = "Electronics",
    in_stock = true,
    quantity = 50;

CREATE products:keyboard SET
    name = "Mechanical Keyboard",
    description = "RGB mechanical keyboard",
    price = 89.99,
    category = "Electronics",
    in_stock = true,
    quantity = 30;

CREATE products:monitor SET
    name = "4K Monitor",
    description = "27-inch 4K display",
    price = 399.99,
    category = "Electronics",
    in_stock = false,
    quantity = 0;

-- Furniture
CREATE products:desk SET
    name = "Standing Desk",
    description = "Adjustable height standing desk",
    price = 199.99,
    category = "Furniture",
    in_stock = true,
    quantity = 10;

CREATE products:chair SET
    name = "Ergonomic Chair",
    description = "Office chair with lumbar support",
    price = 149.99,
    category = "Furniture",
    in_stock = false,
    quantity = 0;

-- Books
CREATE products:rust_book SET
    name = "The Rust Programming Language",
    description = "Official Rust book",
    price = 39.99,
    category = "Books",
    in_stock = true,
    quantity = 100;

CREATE products:surrealdb_guide SET
    name = "SurrealDB Complete Guide",
    description = "Comprehensive SurrealDB guide",
    price = 49.99,
    category = "Books",
    in_stock = true,
    quantity = 75;

-- ----------------------------------------------------------------------------
-- SECTION 8: Sample Data - Orders
-- ----------------------------------------------------------------------------

-- Alice's orders
CREATE orders:order1 SET
    user_id = users:alice,
    product_id = products:laptop,
    quantity = 1,
    total_price = 999.99,
    status = "delivered",
    notes = "Express shipping requested";

CREATE orders:order2 SET
    user_id = users:alice,
    product_id = products:desk,
    quantity = 1,
    total_price = 199.99,
    status = "shipped",
    notes = "Handle with care";

-- Bob's orders
CREATE orders:order3 SET
    user_id = users:bob,
    product_id = products:mouse,
    quantity = 2,
    total_price = 59.98,
    status = "delivered",
    notes = "";

CREATE orders:order4 SET
    user_id = users:bob,
    product_id = products:keyboard,
    quantity = 1,
    total_price = 89.99,
    status = "processing",
    notes = "";

-- Charlie's orders
CREATE orders:order5 SET
    user_id = users:charlie,
    product_id = products:rust_book,
    quantity = 3,
    total_price = 119.97,
    status = "delivered",
    notes = "Gift wrap requested";

-- Eve's orders
CREATE orders:order6 SET
    user_id = users:eve,
    product_id = products:monitor,
    quantity = 1,
    total_price = 399.99,
    status = "pending",
    notes = "Waiting for restock";

-- ----------------------------------------------------------------------------
-- SECTION 9: Sample Data - Reviews
-- ----------------------------------------------------------------------------

-- Reviews for laptop
CREATE reviews:review1 SET
    product_id = products:laptop,
    user_id = users:alice,
    rating = 5,
    comment = "Excellent performance! Perfect for development work.",
    helpful_count = 12;

CREATE reviews:review2 SET
    product_id = products:laptop,
    user_id = users:bob,
    rating = 4,
    comment = "Great laptop but a bit pricey.",
    helpful_count = 5;

-- Reviews for mouse
CREATE reviews:review3 SET
    product_id = products:mouse,
    user_id = users:bob,
    rating = 5,
    comment = "Very comfortable and responsive.",
    helpful_count = 8;

-- Reviews for rust book
CREATE reviews:review4 SET
    product_id = products:rust_book,
    user_id = users:charlie,
    rating = 5,
    comment = "Best programming language book ever!",
    helpful_count = 25;

CREATE reviews:review5 SET
    product_id = products:rust_book,
    user_id = users:eve,
    rating = 4,
    comment = "Comprehensive but challenging for beginners.",
    helpful_count = 10;

-- ----------------------------------------------------------------------------
-- SECTION 10: Verification Queries
-- ----------------------------------------------------------------------------

-- Query 1: Get all active users
-- Expected: 4 users (alice, bob, charlie, eve)
SELECT * FROM users WHERE status = "active";

-- Query 2: Get users by age range
-- Expected: 3 users (bob=25, david=28, alice=30)
SELECT * FROM users WHERE age >= 25 AND age <= 30 ORDER BY age;

-- Query 3: Get all in-stock products
-- Expected: 6 products
SELECT name, category, price, quantity FROM products
WHERE in_stock = true
ORDER BY category, name;

-- Query 4: Get products by category
-- Expected: 4 electronics, 2 furniture, 2 books
SELECT category, count() AS total_products, math::sum(quantity) AS total_quantity
FROM products
GROUP BY category
ORDER BY category;

-- Query 5: Get orders with user and product details (JOIN)
-- Expected: 6 orders with user and product names
SELECT
    order_date,
    user_id.username AS user_name,
    user_id.email AS user_email,
    product_id.name AS product_name,
    quantity,
    total_price,
    status
FROM orders
ORDER BY order_date DESC;

-- Query 6: Get user's order history
-- Expected: Alice's 2 orders
SELECT
    product_id.name AS product,
    quantity,
    total_price,
    status,
    order_date
FROM orders
WHERE user_id = users:alice
ORDER BY order_date DESC;

-- Query 7: Get product reviews with ratings
-- Expected: Laptop (2 reviews), Mouse (1), Rust book (2)
SELECT
    product_id.name AS product_name,
    user_id.username AS reviewer,
    rating,
    comment,
    helpful_count,
    created_at
FROM reviews
ORDER BY helpful_count DESC;

-- Query 8: Get average rating per product
-- Expected: Laptop (4.5), Mouse (5.0), Rust book (4.5)
SELECT
    product_id.name AS product_name,
    math::mean(rating) AS avg_rating,
    count() AS review_count
FROM reviews
GROUP BY product_id
ORDER BY avg_rating DESC;

-- Query 9: Get top-selling products
-- Expected: Rust book (3), Mouse (2), others (1)
SELECT
    product_id.name AS product_name,
    math::sum(quantity) AS total_sold,
    math::sum(total_price) AS revenue
FROM orders
WHERE status != "cancelled"
GROUP BY product_id
ORDER BY total_sold DESC;

-- Query 10: Complex query - Active users with delivered orders
-- Expected: alice (2 items), bob (2 items), charlie (3 items)
SELECT
    user_id.username AS username,
    count() AS completed_orders,
    math::sum(quantity) AS total_items_purchased,
    math::sum(total_price) AS total_spent
FROM orders
WHERE
    status = "delivered"
    AND user_id.status = "active"
GROUP BY user_id
ORDER BY total_spent DESC;

-- ----------------------------------------------------------------------------
-- SECTION 11: Advanced Queries for Testing
-- ----------------------------------------------------------------------------

-- Transaction example - Create order with inventory check
BEGIN TRANSACTION;
    -- Check if product has sufficient quantity
    LET $product = SELECT * FROM products:keyboard WHERE in_stock = true AND quantity > 0;

    -- Create order if product available
    IF $product {
        CREATE orders SET
            user_id = users:alice,
            product_id = products:keyboard,
            quantity = 1,
            total_price = 89.99,
            status = "pending";

        -- Update inventory
        UPDATE products:keyboard SET quantity = quantity - 1;
    };
COMMIT TRANSACTION;

-- Subquery example - Users who purchased electronics
SELECT username, email FROM users
WHERE id IN (
    SELECT user_id FROM orders
    WHERE product_id IN (
        SELECT id FROM products WHERE category = "Electronics"
    )
);

-- Aggregation example - Monthly order statistics
SELECT
    time::month(order_date) AS month,
    count() AS order_count,
    math::sum(total_price) AS monthly_revenue,
    math::mean(total_price) AS avg_order_value
FROM orders
GROUP BY month
ORDER BY month;

-- ----------------------------------------------------------------------------
-- SECTION 12: Cleanup Queries (for testing teardown)
-- ----------------------------------------------------------------------------

-- Remove all reviews (uncomment to use)
-- DELETE reviews;

-- Remove all orders (uncomment to use)
-- DELETE orders;

-- Remove all products (uncomment to use)
-- DELETE products;

-- Remove all users (uncomment to use)
-- DELETE users;

-- Remove entire database (uncomment to use)
-- REMOVE DATABASE test;

-- Remove namespace (uncomment to use)
-- REMOVE NAMESPACE test;

-- ============================================================================
-- END OF TEST DATA
-- ============================================================================

-- Success message
RETURN "Test data loaded successfully";
