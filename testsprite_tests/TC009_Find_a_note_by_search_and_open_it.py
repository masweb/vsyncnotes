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
        
        # -> Click the 'Dev seed' button to load demo data (the button tooltip indicates the demo password is 'dev123').
        # button "Dev seed" title="Cargar datos de demo (contrase"
        elem = page.locator("xpath=/html/body/div/div/div/div[3]/button[2]").nth(0)
        await elem.wait_for(state="visible", timeout=10000)
        await elem.click()
        
        # -> Fill both password fields with the demo password 'dev123' and click 'Crear' to create/unlock the vault (then observe UI for an editor or search UI).
        # password input placeholder="Contraseña"
        elem = page.locator("xpath=/html/body/div/div/div/div/input").nth(0)
        await elem.wait_for(state="visible", timeout=10000)
        await elem.fill("dev123")
        
        # -> Fill both password fields with the demo password 'dev123' and click 'Crear' to create/unlock the vault (then observe UI for an editor or search UI).
        # password input placeholder="Confirmar contraseña"
        elem = page.locator("xpath=/html/body/div/div/div/div[2]/input").nth(0)
        await elem.wait_for(state="visible", timeout=10000)
        await elem.fill("dev123")
        
        # -> Fill both password fields with the demo password 'dev123' and click 'Crear' to create/unlock the vault (then observe UI for an editor or search UI).
        # button "Crear"
        elem = page.locator("xpath=/html/body/div/div/div/div[3]/button").nth(0)
        await elem.wait_for(state="visible", timeout=10000)
        await elem.click()
        
        # --> Test blocked (AST guard fallback)
        raise AssertionError("Test blocked during agent run: " + "TEST BLOCKED The test could not be run \u2014 the app requires Tauri backend functionality that is not available in the browser context, preventing vault creation/unlock and further UI flows. Observations: - The page shows: \"TypeError: Cannot read properties of undefined (reading 'invoke')\" - The UI remains on the 'Crear vault' screen after entering the demo password and clicking 'Crear'")
        await asyncio.sleep(5)
    finally:
        if context:
            await context.close()
        if browser:
            await browser.close()
        if pw:
            await pw.stop()

asyncio.run(run_test())
    