from playwright.sync_api import sync_playwright, expect

def run():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()
        page.goto("http://localhost:3003/")

        # Wait for the navigation bar to be visible
        try:
            nav_bar = page.locator("nav")
            expect(nav_bar).to_be_visible(timeout=30000)
        except Exception as e:
            print(page.content())
            raise e

        # Take a screenshot
        page.screenshot(path="jules-scratch/verification/verification.png")

        browser.close()

if __name__ == "__main__":
    run()
