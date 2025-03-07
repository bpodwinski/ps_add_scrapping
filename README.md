# PrestaShop Addons Scraper & WooCommerce Importer

This project automates the **scraping of PrestaShop Addons** and imports the extracted products into **WooCommerce**. The program is **fully autonomous**, meaning it can fetch, process, and insert products without manual intervention. It efficiently handles large datasets, bypasses scraping protections, and optimizes WooCommerce API interactions.

---

## **Features**
✔️ **Get URLs via `sitemap.xml`** from PrestaShop Addons  
✔️ **Updates URLs to be scraped if more than 7 days old**  
✔️ **Scrapes URLs and creates WooCommerce categories and products in parallel**  
✔️ **Uses FlareSolverr to bypass bot protection**  
✔️ **Fully autonomous process**—no manual intervention required  
✔️ **Stores processed URLs in SQLite to avoid duplicates**  
✔️ **Processes tasks asynchronously with Tokio for better performance**  

---

## **Installation & Configuration**

### **Prerequisites**
Before running this project, ensure you have the following installed:

- **Rust & Cargo** → [Install Rust](https://www.rust-lang.org/)
- **SQLite** → Used as the local database
- **FlareSolverr** → Required to bypass anti-bot protections
- **WordPress with WooCommerce** (API REST must be enabled)

### **Installation**
To extend WooCommerce’s API and allow product/category management, you need to install a custom WordPress plugin.

Clone the repository and build the project:
```sh
git clone https://github.com/bpodwinski/ps_add_scrapping.git
cd ps_add_scrapping
cargo build
```

### **How to Use**
1. **Rename `Settings.toml.example` to `Settings.toml`**  
2. **Place `Settings.toml` in the same directory as the executable**  
3. **Configure it with your settings**

---

## **Usage**
### **Running the Scraper**
```sh
cargo run --release
```

---

## **How It Works**
1. **Extract Sitemap Data**  
   - Reads the **PrestaShop Addons sitemap XML file**  
   - Filters out **non-product URLs**  
   - Stores the filtered URLs in an SQLite database  

2. **Scrape & Process URLs**  
   - **Checks if URLs are older than 7 days** before scraping  
   - Uses **FlareSolverr** to fetch product pages while avoiding bans  
   - Extracts **title, price, description, images, categories**  
   - Checks if the product already exists in WooCommerce  

3. **Insert Data into WooCommerce**  
   - **Finds or creates categories** using WooCommerce API  
   - **Finds or creates products**, attaching them to the right category  
   - Updates SQLite to track processed URLs  

---

## **Performance Optimizations**
**Asynchronous Processing:** Uses **Tokio & Futures** for parallel execution!  
**Batch URL Processing:** Handles URLs in configurable batches to optimize speed  
**Delayed Requests:** Implements randomized delays to prevent detection  
**Database Optimization:** Uses SQLite to track changes and avoid reprocessing  
