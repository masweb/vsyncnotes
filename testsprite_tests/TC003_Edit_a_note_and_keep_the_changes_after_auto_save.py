import asyncio
import re
from playwright import async_api
from playwright.async_api import expect

async def run_test():
    pw = None
    browser = None
    context = None

    try:
        pw = await async_api.async_playwright().start()
        browser = await pw.chromium.launch(
            headless=True,
            args=[
                "--window-size=1280,720",
                "--disable-dev-shm-usage",
                "--ipc=host",
                "--single-process"
            ],
        )
        context = await browser.new_context()
        context.set_default_timeout(15000)
        page = await context.new_page()
        # -> navigate
        await page.goto("http://localhost:1420")
        try:
            await page.wait_for_load_state("domcontentloaded", timeout=5000)
        except Exception:
            pass
        
        # -> Click the 'Dev seed' button to load demo data so the demo vault can be unlocked with password 'dev123'.
        # button "Dev seed" title="Cargar datos de demo (contrase"
        elem = page.locator("xpath=/html/body/div/div/div/div[3]/button[2]").nth(0)
        await elem.wait_for(state="visible", timeout=10000)
        await elem.click()
        
        # -> Fill the master password and confirmation fields with 'dev123' and click the 'Crear' button to unlock/open the demo vault.
        # password input placeholder="Contraseña"
        elem = page.locator("xpath=/html/body/div/div/div/div/input").nth(0)
        await elem.wait_for(state="visible", timeout=10000)
        await elem.fill("dev123")
        
        # -> Fill the master password and confirmation fields with 'dev123' and click the 'Crear' button to unlock/open the demo vault.
        # password input placeholder="Confirmar contraseña"
        elem = page.locator("xpath=/html/body/div/div/div/div[2]/input").nth(0)
        await elem.wait_for(state="visible", timeout=10000)
        await elem.fill("dev123")
        
        # -> Fill the master password and confirmation fields with 'dev123' and click the 'Crear' button to unlock/open the demo vault.
        # button "Crear"
        elem = page.locator("xpath=/html/body/div/div/div/div[3]/button").nth(0)
        await elem.wait_for(state="visible", timeout=10000)
        await elem.click()
        
        # --> Test blocked (AST guard fallback)
        raise AssertionError("Test blocked during agent run: " + "TEST BLOCKED The test could not be run \u2014 the app requires Tauri backend calls (invoke) which are not available in the browser environment, preventing demo data from being loaded and the vault from being opened. Observations: - The page shows a runtime error: \"TypeError: Cannot read properties of undefined (reading 'invoke')\" - After clicking 'Dev seed' and 'Crear', the UI remained on the 'Crear...")
        await asyncio.sleep(5)
    finally:
        if context:
            await context.close()
        if browser:
            await browser.close()
        if pw:
            await pw.stop()

asyncio.run(run_test())
    