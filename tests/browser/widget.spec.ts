import { test, expect } from '@playwright/test';
import { defaultSettings } from '../../src/lib/types/contracts';

test('widget layouts fit at all supported scale factors', async ({
  browser,
}) => {
  for (const scale of [1, 1.25, 1.5, 2]) {
    const context = await browser.newContext({
      viewport: { width: 210, height: 105 },
      deviceScaleFactor: scale,
    });
    const page = await context.newPage();
    await page.addInitScript((settings) => {
      const state = {
        schemaVersion: 1,
        revision: 1,
        settings,
        timer: {
          session: 'focus',
          status: 'idle',
          durationMs: 1500000,
          remainingMs: 1500000,
          completedFocus: 0,
          taskId: null,
          wallDeadlineMs: null,
        },
        tasks: {
          lists: [{ id: 'today', name: 'Today' }],
          tasks: Array.from({ length: 20 }, (_, i) => ({
            id: `${i}`,
            listId: 'today',
            title:
              i === 0
                ? 'A very long task title that should truncate cleanly inside this compact widget'
                : `Focus task ${i + 1}`,
            completed: i < 3,
            createdAt: 1,
            updatedAt: 1,
            completedAt: i < 3 ? 1 : null,
            order: i,
          })),
          activeListId: 'today',
        },
        window: { expanded: false, position: null },
        history: [],
        customAudio: false,
        recovery: null,
      };
      // Browser tests exercise production components with a fake transport, not a second timer engine.
      Object.assign(window, {
        __TAURI_INTERNALS__: {
          transformCallback: () => 1,
          unregisterCallback: () => {},
          invoke: async (command: string, args: { expanded?: boolean }) => {
            if (command.startsWith('plugin:')) return 1;
            if (command === 'window_set_mode') {
              state.window.expanded = !!args.expanded;
              state.revision++;
            }
            return structuredClone(state);
          },
        },
      });
    }, defaultSettings);
    await page.goto('/');
    await expect(page.getByRole('timer')).toHaveText('25:00');
    expect(await page.locator('.dot').count()).toBe(10);
    const compactBounds = await page.locator('.timer-dots').boundingBox();
    expect(compactBounds!.y + compactBounds!.height).toBeLessThanOrEqual(105);
    if (scale === 2)
      await page.screenshot({ path: 'docs/screenshots/compact.png' });
    await page.getByRole('button', { name: 'Expand widget' }).click();
    await page.setViewportSize({ width: 350, height: 520 });
    await expect(
      page.getByRole('button', { name: 'Open settings' }),
    ).toBeVisible();
    const input = await page
      .getByRole('textbox', { name: 'New task' })
      .boundingBox();
    expect(input!.y + input!.height).toBeLessThan(520);
    expect(
      await page
        .locator('.task-list')
        .evaluate((el) => el.scrollHeight > el.clientHeight),
    ).toBe(true);
    expect(
      await page
        .locator('.widget-shell')
        .evaluate((el) => el.scrollWidth <= el.clientWidth),
    ).toBe(true);
    if (scale === 2)
      await page.screenshot({ path: 'docs/screenshots/expanded.png' });
    await page.getByRole('button', { name: 'Open settings' }).click();
    await expect(
      page.getByRole('button', { name: 'Save changes' }),
    ).toBeVisible();
    const save = await page
      .getByRole('button', { name: 'Save changes' })
      .boundingBox();
    expect(save!.y + save!.height).toBeLessThan(520);
    if (scale === 2)
      await page.screenshot({ path: 'docs/screenshots/settings.png' });
    await context.close();
  }
});
