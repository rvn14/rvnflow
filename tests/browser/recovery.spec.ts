import { test, expect } from '@playwright/test';
test('startup recovery stays actionable without replacing saved data', async ({
  page,
}) => {
  await page.setViewportSize({ width: 350, height: 240 });
  await page.addInitScript(() =>
    Object.assign(window, {
      __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener: () => {} },
      __TAURI_INTERNALS__: {
        transformCallback: () => 1,
        unregisterCallback: () => {},
        invoke: async (command: string) =>
          command.startsWith('plugin:')
            ? 1
            : command === 'get_startup_status'
              ? {
                  ready: false,
                  loading: false,
                  error:
                    'Saved data could not be read. Your tasks are preserved. Repair the file and retry recovery.',
                }
              : undefined,
      },
    }),
  );
  await page.goto('/');
  await expect(page.getByText(/Your tasks are preserved/)).toBeVisible();
  await expect(
    page.getByRole('button', { name: 'Retry recovery' }),
  ).toBeEnabled();
  await page.getByRole('button', { name: 'Retry recovery' }).click();
  await expect(page.getByText(/Your tasks are preserved/)).toBeVisible();
});
