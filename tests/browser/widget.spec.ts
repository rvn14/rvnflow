import { test, expect, type Page } from '@playwright/test';
import { defaultSettings } from '../../src/lib/types/contracts';

async function mockNative(page: Page, count: number) {
  await page.addInitScript(
    ({ settings, count }) => {
      const state = {
        schemaVersion: 3,
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
          tasks: Array.from({ length: count }, (_, i) => ({
            id: `${i}`,
            listId: 'today',
            title:
              i === 0
                ? 'A very long task title that should truncate cleanly inside this compact widget'
                : `Focus task ${i + 1}`,
            completed: false,
            createdAt: 1,
            updatedAt: 1,
            completedAt: null,
            order: i,
          })),
          activeListId: 'today',
        },
        window: { expanded: false, position: null },
        history: [],
        customAudio: false,
        recovery: null,
      };
      Object.assign(window, {
        __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener: () => {} },
        __TAURI_INTERNALS__: {
          transformCallback: () => 1,
          unregisterCallback: () => {},
          invoke: async (
            command: string,
            args: {
              expanded?: boolean;
              change?: { action: string; title: string; id: string };
            },
          ) => {
            if (command.startsWith('plugin:')) return 1;
            if (command === 'get_startup_status')
              return { ready: true, loading: false, error: null };
            if (command === 'frontend_ready') return;
            if (command === 'window_set_mode')
              state.window.expanded = !!args.expanded;
            if (command === 'task_action' && args.change) {
              if (args.change.action === 'create') {
                if (args.change.title === 'Fail to save')
                  throw 'Unable to save: disk full';
                state.tasks.tasks.push({
                  id: `new-${state.revision}`,
                  listId: 'today',
                  title: args.change.title,
                  completed: false,
                  createdAt: 1,
                  updatedAt: 1,
                  completedAt: null,
                  order: state.tasks.tasks.length,
                });
              }
              if (args.change.action === 'update') {
                const task = state.tasks.tasks.find(
                  (t) => t.id === args.change!.id,
                );
                if (task) task.title = args.change.title;
              }
            }
            state.revision++;
            return structuredClone(state);
          },
        },
      });
    },
    { settings: defaultSettings, count },
  );
}

for (const count of [0, 1, 10, 50]) {
  test(`${count} tasks: composer stays visible at all supported scales`, async ({
    browser,
  }) => {
    for (const scale of [1, 1.25, 1.5, 2]) {
      const context = await browser.newContext({
        viewport: { width: 210, height: 105 },
        deviceScaleFactor: scale,
      });
      const page = await context.newPage();
      await mockNative(page, count);
      await page.goto('/');
      await expect(page.getByRole('timer')).toHaveText('25:00');
      expect(
        await page
          .locator('html')
          .evaluate((el) => getComputedStyle(el).backgroundColor),
      ).toBe('rgba(0, 0, 0, 0)');
      expect(
        await page
          .locator('.widget-shell')
          .evaluate((el) => getComputedStyle(el).boxShadow),
      ).toBe('none');
      await expect(
        page.getByRole('group', { name: 'Session type' }),
      ).toHaveCount(0);
      expect(await page.locator('.dot').count()).toBe(10);
      const compact = await page.locator('.timer-dots').boundingBox();
      expect(compact!.y + compact!.height).toBeLessThanOrEqual(105);
      if (count === 10 && scale === 2)
        await page.screenshot({
          path: 'docs/screenshots/compact.png',
          omitBackground: true,
        });
      await page.getByRole('button', { name: 'Expand widget' }).click();
      await page.setViewportSize({ width: 350, height: 560 });
      const composer = page.locator('.task-composer');
      await expect(composer).toBeVisible();
      const input = await composer.boundingBox();
      const list = await page.locator('.task-list').boundingBox();
      expect(list!.y + list!.height).toBeLessThanOrEqual(input!.y);
      expect(input!.y + input!.height).toBeLessThan(560);
      expect(
        await page
          .locator('.widget-shell')
          .evaluate((el) => el.scrollWidth <= el.clientWidth),
      ).toBe(true);
      if (count >= 10)
        expect(
          await page
            .locator('.task-list')
            .evaluate((el) => el.scrollHeight > el.clientHeight),
        ).toBe(true);
      if (count === 10 && scale === 2)
        await page.screenshot({
          path: 'docs/screenshots/expanded.png',
          omitBackground: true,
        });
      await page
        .getByRole('textbox', { name: 'New task', exact: true })
        .fill('Another task');
      await page
        .getByRole('textbox', { name: 'New task', exact: true })
        .press('Enter');
      await expect(
        page.getByRole('textbox', { name: 'New task', exact: true }),
      ).toHaveValue('');
      await page.getByRole('button', { name: 'Dismiss notification' }).click();
      await page
        .getByRole('button', { name: 'Open settings', exact: true })
        .click();
      const save = await page
        .getByRole('button', { name: 'Save changes' })
        .boundingBox();
      expect(save!.y + save!.height).toBeLessThan(560);
      if (count === 10 && scale === 2)
        await page.screenshot({
          path: 'docs/screenshots/settings.png',
          omitBackground: true,
        });
      await context.close();
    }
  });
}

test('failed task creation retains text; task editing stays inside the row', async ({
  page,
}) => {
  await mockNative(page, 50);
  await page.goto('/');
  await page.getByRole('button', { name: 'Expand widget' }).click();
  await page.setViewportSize({ width: 350, height: 560 });
  const input = page.getByRole('textbox', { name: 'New task', exact: true });
  await input.fill('Fail to save');
  await input.press('Enter');
  await expect(input).toHaveValue('Fail to save');
  await expect(page.getByRole('alert')).toContainText('disk full');
  await page.getByRole('button', { name: 'Dismiss notification' }).click();
  await page
    .getByRole('button', { name: 'More actions for Focus task 2', exact: true })
    .click();
  await page
    .getByRole('button', { name: 'Edit Focus task 2', exact: true })
    .click();
  const edit = page.getByRole('textbox', { name: 'Edit task title' });
  await edit.fill('Edited task');
  await edit.press('Enter');
  await expect(page.getByText('Edited task', { exact: true })).toBeVisible();
  const list = await page.locator('.task-list').boundingBox();
  const composer = await page.locator('.task-composer').boundingBox();
  expect(list!.y + list!.height).toBeLessThanOrEqual(composer!.y);
});
