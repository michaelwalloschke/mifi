import { Effect, Match as M, Schema as S } from 'effect'
import { Command, Runtime } from 'foldkit'
import { Document, html } from 'foldkit/html'
import { m } from 'foldkit/message'

import { invoke } from '@tauri-apps/api/core'

// MODEL

export const Account = S.Struct({
  id: S.Number,
  name: S.String,
})
export type Account = typeof Account.Type

export const Split = S.Struct({
  category_name: S.NullOr(S.String),
  amount_cents: S.Number,
  category_source: S.String,
})
export type Split = typeof Split.Type

export const Transaction = S.Struct({
  id: S.Number,
  booking_date: S.String,
  account_name: S.String,
  counterparty: S.String,
  purpose: S.String,
  amount_cents: S.Number,
  is_transfer: S.Boolean,
  splits: S.Array(Split),
})
export type Transaction = typeof Transaction.Type

export const MonthlyOverview = S.Struct({
  month: S.String,
  einnahmen_cents: S.Number,
  ausgaben_cents: S.Number,
  sparquote_percent: S.Number,
  puffer_cents: S.Number,
})
export type MonthlyOverview = typeof MonthlyOverview.Type

export const Overview = S.Struct({
  current: MonthlyOverview,
  previous: MonthlyOverview,
  sparkline: S.Array(MonthlyOverview),
})
export type Overview = typeof Overview.Type

export const Category = S.Struct({
  id: S.Number,
  parent_id: S.NullOr(S.Number),
  name: S.String,
  kind: S.String,
})
export type Category = typeof Category.Type

export const Subcategory = S.Struct({
  id: S.Number,
  name: S.String,
  spent_cents: S.Number,
})
export type Subcategory = typeof Subcategory.Type

export const Contract = S.Struct({
  id: S.Number,
  normalized_counterparty: S.String,
  direction: S.String,
  expected_amount_cents: S.Number,
  interval: S.String,
  status: S.String,
})
export type Contract = typeof Contract.Type

export const CategoryDetail = S.Struct({
  id: S.Number,
  name: S.String,
  kind: S.String,
  month: S.String,
  spent_cents: S.Number,
  subcategories: S.Array(Subcategory),
  contracts: S.Array(Contract),
})
export type CategoryDetail = typeof CategoryDetail.Type

export const BudgetRow = S.Struct({
  category_id: S.Number,
  category_name: S.String,
  parent_id: S.NullOr(S.Number),
  target_cents: S.Number,
  spent_cents: S.Number,
  state: S.String,
})
export type BudgetRow = typeof BudgetRow.Type

export const BudgetOverview = S.Struct({
  month: S.String,
  rows: S.Array(BudgetRow),
  unbudgeted_expense_cents: S.Number,
})
export type BudgetOverview = typeof BudgetOverview.Type

export const Screen = S.Literals(['Uebersicht', 'Transaktionen', 'Kategorien', 'Budget'])
export type Screen = typeof Screen.Type

export const Model = S.Struct({
  screen: Screen,
  accounts: S.Array(Account),
  transactions: S.Array(Transaction),
  selectedAccountId: S.NullOr(S.Number),
  search: S.String,
  loading: S.Boolean,
  error: S.NullOr(S.String),
  overview: S.NullOr(Overview),
  overviewError: S.NullOr(S.String),
  categories: S.Array(Category),
  categoriesError: S.NullOr(S.String),
  selectedCategoryId: S.NullOr(S.Number),
  categoryDetail: S.NullOr(CategoryDetail),
  categoryDetailError: S.NullOr(S.String),
  budgetOverview: S.NullOr(BudgetOverview),
  budgetOverviewError: S.NullOr(S.String),
  budgetFormCategoryId: S.NullOr(S.Number),
  budgetFormAmount: S.String,
  budgetFormError: S.NullOr(S.String),
})
export type Model = typeof Model.Type

// MESSAGE

export const FetchedAccounts = m('FetchedAccounts', { accounts: S.Array(Account) })
export const FailedFetchAccounts = m('FailedFetchAccounts', { error: S.String })
export const FetchedTransactions = m('FetchedTransactions', { transactions: S.Array(Transaction) })
export const FailedFetchTransactions = m('FailedFetchTransactions', { error: S.String })
export const SelectedAccount = m('SelectedAccount', { accountId: S.NullOr(S.Number) })
export const TypedSearch = m('TypedSearch', { query: S.String })
export const FetchedOverview = m('FetchedOverview', { overview: Overview })
export const FailedFetchOverview = m('FailedFetchOverview', { error: S.String })
export const ClickedNavItem = m('ClickedNavItem', { screen: Screen })
export const FetchedCategories = m('FetchedCategories', { categories: S.Array(Category) })
export const FailedFetchCategories = m('FailedFetchCategories', { error: S.String })
export const SelectedCategory = m('SelectedCategory', { categoryId: S.Number })
export const FetchedCategoryDetail = m('FetchedCategoryDetail', { detail: CategoryDetail })
export const FailedFetchCategoryDetail = m('FailedFetchCategoryDetail', { error: S.String })
export const FetchedBudgetOverview = m('FetchedBudgetOverview', { overview: BudgetOverview })
export const FailedFetchBudgetOverview = m('FailedFetchBudgetOverview', { error: S.String })
export const TypedBudgetFormCategory = m('TypedBudgetFormCategory', { categoryId: S.NullOr(S.Number) })
export const TypedBudgetFormAmount = m('TypedBudgetFormAmount', { amount: S.String })
export const ClickedSetBudgetTarget = m('ClickedSetBudgetTarget')
export const SetBudgetTargetSucceeded = m('SetBudgetTargetSucceeded')
export const SetBudgetTargetFailed = m('SetBudgetTargetFailed', { error: S.String })

export const Message = S.Union([
  FetchedAccounts,
  FailedFetchAccounts,
  FetchedTransactions,
  FailedFetchTransactions,
  SelectedAccount,
  TypedSearch,
  FetchedOverview,
  FailedFetchOverview,
  ClickedNavItem,
  FetchedCategories,
  FailedFetchCategories,
  SelectedCategory,
  FetchedCategoryDetail,
  FailedFetchCategoryDetail,
  FetchedBudgetOverview,
  FailedFetchBudgetOverview,
  TypedBudgetFormCategory,
  TypedBudgetFormAmount,
  ClickedSetBudgetTarget,
  SetBudgetTargetSucceeded,
  SetBudgetTargetFailed,
])
export type Message = typeof Message.Type

// COMMANDS

export const FetchAccounts = Command.define('FetchAccounts', FetchedAccounts, FailedFetchAccounts)(
  Effect.tryPromise(() => invoke<ReadonlyArray<Account>>('list_accounts')).pipe(
    Effect.match({
      onSuccess: accounts => FetchedAccounts({ accounts }),
      onFailure: error => FailedFetchAccounts({ error: String(error) }),
    }),
  ),
)

export const FetchTransactions = Command.define(
  'FetchTransactions',
  { accountId: S.NullOr(S.Number), search: S.String },
  FetchedTransactions,
  FailedFetchTransactions,
)(({ accountId, search }) =>
  Effect.tryPromise(() =>
    invoke<ReadonlyArray<Transaction>>('list_transactions', {
      accountId,
      search: search.length > 0 ? search : null,
    }),
  ).pipe(
    Effect.match({
      onSuccess: transactions => FetchedTransactions({ transactions }),
      onFailure: error => FailedFetchTransactions({ error: String(error) }),
    }),
  ),
)

export const FetchOverview = Command.define('FetchOverview', FetchedOverview, FailedFetchOverview)(
  Effect.tryPromise(() => invoke<Overview>('get_overview', { month: null })).pipe(
    Effect.match({
      onSuccess: overview => FetchedOverview({ overview }),
      onFailure: error => FailedFetchOverview({ error: String(error) }),
    }),
  ),
)

export const FetchCategories = Command.define('FetchCategories', FetchedCategories, FailedFetchCategories)(
  Effect.tryPromise(() => invoke<ReadonlyArray<Category>>('list_categories')).pipe(
    Effect.match({
      onSuccess: categories => FetchedCategories({ categories }),
      onFailure: error => FailedFetchCategories({ error: String(error) }),
    }),
  ),
)

export const FetchCategoryDetail = Command.define(
  'FetchCategoryDetail',
  { categoryId: S.Number },
  FetchedCategoryDetail,
  FailedFetchCategoryDetail,
)(({ categoryId }) =>
  Effect.tryPromise(() => invoke<CategoryDetail>('get_category_detail', { categoryId, month: null })).pipe(
    Effect.match({
      onSuccess: detail => FetchedCategoryDetail({ detail }),
      onFailure: error => FailedFetchCategoryDetail({ error: String(error) }),
    }),
  ),
)

export const FetchBudgetOverview = Command.define(
  'FetchBudgetOverview',
  FetchedBudgetOverview,
  FailedFetchBudgetOverview,
)(
  Effect.tryPromise(() => invoke<BudgetOverview>('get_budget_overview', { month: null })).pipe(
    Effect.match({
      onSuccess: overview => FetchedBudgetOverview({ overview }),
      onFailure: error => FailedFetchBudgetOverview({ error: String(error) }),
    }),
  ),
)

const currentMonthString = (): string => new Date().toISOString().slice(0, 7)

export const SetBudgetTarget = Command.define(
  'SetBudgetTarget',
  { categoryId: S.Number, amountCents: S.Number },
  SetBudgetTargetSucceeded,
  SetBudgetTargetFailed,
)(({ categoryId, amountCents }) =>
  Effect.tryPromise(() =>
    invoke('set_budget_target', { categoryId, amountCents, effectiveFromMonth: currentMonthString() }),
  ).pipe(
    Effect.match({
      onSuccess: () => SetBudgetTargetSucceeded(),
      onFailure: error => SetBudgetTargetFailed({ error: String(error) }),
    }),
  ),
)

// UPDATE

export const update = (
  model: Model,
  message: Message,
): readonly [Model, ReadonlyArray<Command.Command<Message>>] =>
  M.value(message).pipe(
    M.withReturnType<readonly [Model, ReadonlyArray<Command.Command<Message>>]>(),
    M.tagsExhaustive({
      FetchedAccounts: ({ accounts }) => [{ ...model, accounts }, []],
      FailedFetchAccounts: ({ error }) => [{ ...model, error }, []],
      FetchedTransactions: ({ transactions }) => [{ ...model, transactions, loading: false }, []],
      FailedFetchTransactions: ({ error }) => [{ ...model, error, loading: false }, []],
      SelectedAccount: ({ accountId }) => {
        const next = { ...model, selectedAccountId: accountId, loading: true }
        return [next, [FetchTransactions({ accountId, search: model.search })]]
      },
      TypedSearch: ({ query }) => {
        const next = { ...model, search: query, loading: true }
        return [next, [FetchTransactions({ accountId: model.selectedAccountId, search: query })]]
      },
      FetchedOverview: ({ overview }) => [{ ...model, overview }, []],
      FailedFetchOverview: ({ error }) => [{ ...model, overviewError: error }, []],
      ClickedNavItem: ({ screen }) => [{ ...model, screen }, []],
      FetchedCategories: ({ categories }) => {
        const next = { ...model, categories }
        const firstParent = categories.find(c => c.parent_id === null)
        if (model.selectedCategoryId === null && firstParent) {
          return [{ ...next, selectedCategoryId: firstParent.id }, [FetchCategoryDetail({ categoryId: firstParent.id })]]
        }
        return [next, []]
      },
      FailedFetchCategories: ({ error }) => [{ ...model, categoriesError: error }, []],
      SelectedCategory: ({ categoryId }) => [
        { ...model, selectedCategoryId: categoryId },
        [FetchCategoryDetail({ categoryId })],
      ],
      FetchedCategoryDetail: ({ detail }) => [{ ...model, categoryDetail: detail }, []],
      FailedFetchCategoryDetail: ({ error }) => [{ ...model, categoryDetailError: error }, []],
      FetchedBudgetOverview: ({ overview }) => [{ ...model, budgetOverview: overview }, []],
      FailedFetchBudgetOverview: ({ error }) => [{ ...model, budgetOverviewError: error }, []],
      TypedBudgetFormCategory: ({ categoryId }) => [{ ...model, budgetFormCategoryId: categoryId }, []],
      TypedBudgetFormAmount: ({ amount }) => [{ ...model, budgetFormAmount: amount }, []],
      ClickedSetBudgetTarget: () => {
        const amountCents = parseEuroToCents(model.budgetFormAmount)
        if (model.budgetFormCategoryId === null || amountCents === null) {
          return [{ ...model, budgetFormError: 'Kategorie und Betrag wählen' }, []]
        }
        return [
          { ...model, budgetFormError: null },
          [SetBudgetTarget({ categoryId: model.budgetFormCategoryId, amountCents })],
        ]
      },
      SetBudgetTargetSucceeded: () => [
        { ...model, budgetFormAmount: '', budgetFormError: null },
        [FetchBudgetOverview()],
      ],
      SetBudgetTargetFailed: ({ error }) => [{ ...model, budgetFormError: error }, []],
    }),
  )

// INIT

export const init: Runtime.ApplicationInit<Model, Message> = () => [
  {
    screen: 'Transaktionen',
    accounts: [],
    transactions: [],
    selectedAccountId: null,
    search: '',
    loading: true,
    error: null,
    overview: null,
    overviewError: null,
    categories: [],
    categoriesError: null,
    selectedCategoryId: null,
    categoryDetail: null,
    categoryDetailError: null,
    budgetOverview: null,
    budgetOverviewError: null,
    budgetFormCategoryId: null,
    budgetFormAmount: '',
    budgetFormError: null,
  },
  [
    FetchAccounts(),
    FetchTransactions({ accountId: null, search: '' }),
    FetchOverview(),
    FetchCategories(),
    FetchBudgetOverview(),
  ],
]

// VIEW HELPERS

const parseEuroToCents = (input: string): number | null => {
  const normalized = input.trim().replace(',', '.')
  if (normalized === '' || Number.isNaN(Number(normalized))) return null
  return Math.round(Number(normalized) * 100)
}

const formatAmountCents = (cents: number): string => {
  const euros = cents / 100
  const formatted = new Intl.NumberFormat('de-DE', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(Math.abs(euros))
  return `${euros < 0 ? '−' : '+'}${formatted} €`
}

const formatBookingDate = (isoDate: string): string => {
  const [, month, day] = isoDate.split('-')
  const monthNames = ['Jan', 'Feb', 'Mär', 'Apr', 'Mai', 'Jun', 'Jul', 'Aug', 'Sep', 'Okt', 'Nov', 'Dez']
  return `${Number(day)}. ${monthNames[Number(month) - 1]}`
}

const groupByBookingDate = (
  transactions: ReadonlyArray<Transaction>,
): ReadonlyArray<readonly [string, ReadonlyArray<Transaction>]> => {
  const groups: Array<[string, Transaction[]]> = []
  for (const transaction of transactions) {
    const last = groups.at(-1)
    if (last && last[0] === transaction.booking_date) {
      last[1].push(transaction)
    } else {
      groups.push([transaction.booking_date, [transaction]])
    }
  }
  return groups
}

// VIEW

const NAV_ITEMS: ReadonlyArray<{ label: string; screen: Screen | null }> = [
  { label: 'Übersicht', screen: 'Uebersicht' },
  { label: 'Transaktionen', screen: 'Transaktionen' },
  { label: 'Kategorien', screen: 'Kategorien' },
  { label: 'Budget', screen: 'Budget' },
  { label: 'Verträge', screen: null },
  { label: 'Vermögen', screen: null },
  { label: 'Konten', screen: null },
]

const sidebar = (h: ReturnType<typeof html<Message>>, model: Model) =>
  h.div(
    [h.Class('w-[216px] shrink-0 flex flex-col justify-between border-r border-black/10 dark:border-white/10 p-4')],
    [
      h.div(
        [],
        [
          h.div([h.Class('text-lg font-semibold px-2 mb-6')], ['mifi']),
          h.nav(
            [h.Class('flex flex-col gap-1')],
            NAV_ITEMS.map(({ label, screen }) =>
              h.div(
                [
                  h.Class(
                    screen === model.screen
                      ? 'px-2 py-1.5 rounded-[10px] bg-black/5 dark:bg-white/10 font-medium cursor-pointer'
                      : screen === null
                        ? 'px-2 py-1.5 rounded-[10px] text-black/40 dark:text-white/40'
                        : 'px-2 py-1.5 rounded-[10px] text-black/60 dark:text-white/60 cursor-pointer hover:bg-black/5 dark:hover:bg-white/5',
                  ),
                  ...(screen !== null ? [h.OnClick(ClickedNavItem({ screen }))] : []),
                ],
                [label],
              ),
            ),
          ),
        ],
      ),
      h.div([h.Class('text-xs text-black/40 dark:text-white/40 px-2')], ['Konten & Sync — nicht implementiert']),
    ],
  )

const splitRow = (h: ReturnType<typeof html<Message>>, split: Split) =>
  h.tr(
    [],
    [
      h.td([h.Class('py-1 pl-8 text-sm text-black/60 dark:text-white/60')], [split.category_name ?? 'Ohne Kategorie']),
      h.td([h.Class('py-1 text-sm text-black/40 dark:text-white/40')], [split.category_source === 'auto' ? 'auto' : 'manuell']),
      h.td([], []),
      h.td([h.Class('py-1 text-right tabular-nums text-sm')], [formatAmountCents(split.amount_cents)]),
    ],
  )

const transactionRow = (h: ReturnType<typeof html<Message>>, transaction: Transaction) => {
  const rows = [
    h.tr(
      [],
      [
        h.td(
          [h.Class('py-1.5')],
          [
            h.div([h.Class('font-medium')], [transaction.counterparty]),
            h.div([h.Class('text-sm text-black/40 dark:text-white/40')], [transaction.purpose]),
          ],
        ),
        h.td([h.Class('py-1.5 text-sm text-black/60 dark:text-white/60')], [transaction.account_name]),
        h.td(
          [h.Class('py-1.5 text-sm')],
          [
            transaction.is_transfer
              ? '⇄ Umbuchung — nicht in Auswertungen'
              : transaction.splits.length > 1
                ? `Split (${transaction.splits.length})`
                : (transaction.splits[0]?.category_name ?? 'Ohne Kategorie'),
          ],
        ),
        h.td([h.Class('py-1.5 text-right tabular-nums font-medium')], [formatAmountCents(transaction.amount_cents)]),
      ],
    ),
  ]
  if (!transaction.is_transfer && transaction.splits.length > 1) {
    rows.push(...transaction.splits.map(split => splitRow(h, split)))
  }
  return rows
}

// Hand-rolled SVG sparkline — no charting library (SPEC.md §13).
const sparkline = (h: ReturnType<typeof html<Message>>, values: ReadonlyArray<number>, dotColor: string) => {
  const width = 88
  const height = 28
  const min = Math.min(...values)
  const max = Math.max(...values)
  const range = max - min || 1
  const points = values
    .map((value, i) => {
      const x = (i / Math.max(values.length - 1, 1)) * width
      const y = height - ((value - min) / range) * height
      return `${x.toFixed(1)},${y.toFixed(1)}`
    })
    .join(' ')
  const last = points.split(' ').at(-1) ?? '0,0'
  const [lastX = '0', lastY = '0'] = last.split(',')

  return h.svg(
    [h.ViewBox(`0 0 ${width} ${height}`), h.Width(String(width)), h.Height(String(height))],
    [
      h.polyline(
        [h.Points(points), h.Fill('none'), h.Stroke('currentColor'), h.StrokeWidth('1.5'), h.Class('text-black/20 dark:text-white/20')],
        [],
      ),
      h.circle([h.Cx(lastX), h.Cy(lastY), h.Rx('2.5'), h.Ry('2.5'), h.Fill(dotColor)], []),
    ],
  )
}

const formatPercent = (value: number): string => `${value >= 0 ? '' : '−'}${Math.abs(Math.round(value))} %`

const statTile = (
  h: ReturnType<typeof html<Message>>,
  label: string,
  value: string,
  delta: string | null,
  sparklineValues: ReadonlyArray<number>,
  dotColor: string,
) =>
  h.div(
    [h.Class('rounded-[10px] border border-black/10 dark:border-white/10 p-4 flex flex-col gap-1')],
    [
      h.div([h.Class('text-sm text-black/40 dark:text-white/40')], [label]),
      h.div([h.Class('text-2xl font-semibold tabular-nums')], [value]),
      delta ? h.div([h.Class('text-xs text-black/40 dark:text-white/40')], [delta]) : h.empty,
      h.div([h.Class('mt-1')], [sparkline(h, sparklineValues, dotColor)]),
    ],
  )

const uebersichtScreen = (h: ReturnType<typeof html<Message>>, model: Model) => {
  if (model.overviewError) {
    return h.div([h.Class('flex-1 p-8')], [h.div([h.Class('text-red-600')], [model.overviewError])])
  }
  if (!model.overview) {
    return h.div([h.Class('flex-1 p-8 text-black/40 dark:text-white/40')], ['Lädt …'])
  }

  const { current, previous, sparkline: series } = model.overview
  const ausgabenDelta = current.ausgaben_cents - previous.ausgaben_cents
  const sparquoteDelta = current.sparquote_percent - previous.sparquote_percent

  return h.div(
    [h.Class('flex-1 p-8 overflow-y-auto')],
    [
      h.div(
        [],
        [
          h.div([h.Class('text-2xl font-semibold')], [`Übersicht · ${current.month}`]),
          h.div([h.Class('text-sm text-black/40 dark:text-white/40 mb-6')], ['Einnahmen → Kategorien → Sparziele']),
        ],
      ),
      h.div(
        [h.Class('grid grid-cols-4 gap-4')],
        [
          statTile(h, 'Einnahmen', formatAmountCents(current.einnahmen_cents), null, series.map(m => m.einnahmen_cents), '#3b82f6'),
          statTile(
            h,
            'Ausgaben',
            formatAmountCents(-current.ausgaben_cents),
            `${ausgabenDelta >= 0 ? '+' : '−'}${formatAmountCents(Math.abs(ausgabenDelta)).replace(/^[+−]/, '')} ggü. ${previous.month}`,
            series.map(m => m.ausgaben_cents),
            '#f97316',
          ),
          statTile(
            h,
            'Sparquote',
            formatPercent(current.sparquote_percent),
            `${sparquoteDelta >= 0 ? '↑' : '↓'} ggü. ${previous.month}`,
            series.map(m => m.sparquote_percent),
            '#22c55e',
          ),
          statTile(h, 'Puffer übrig', formatAmountCents(current.puffer_cents), null, series.map(m => m.puffer_cents), '#eab308'),
        ],
      ),
    ],
  )
}

const transaktionenScreen = (h: ReturnType<typeof html<Message>>, model: Model) =>
  h.div(
    [h.Class('flex-1 p-8 overflow-y-auto')],
    [
      h.div(
        [h.Class('flex items-start justify-between mb-6')],
        [
          h.div(
            [],
            [
              h.div([h.Class('text-2xl font-semibold')], ['Transaktionen']),
              h.div([h.Class('text-sm text-black/40 dark:text-white/40')], ['Splits und Umbuchungen inline']),
            ],
          ),
          h.div(
            [h.Class('flex gap-2')],
            [
              h.select(
                [
                  h.Class('rounded-[10px] border border-black/10 dark:border-white/20 px-3 py-1.5 bg-transparent'),
                  h.OnChange(value => SelectedAccount({ accountId: value === '' ? null : Number(value) })),
                ],
                [
                  h.option([h.Value('')], ['Alle Konten']),
                  ...model.accounts.map(account =>
                    h.option(
                      [h.Value(String(account.id)), h.Selected(model.selectedAccountId === account.id)],
                      [account.name],
                    ),
                  ),
                ],
              ),
              h.input([
                h.Class('rounded-[10px] border border-black/10 dark:border-white/20 px-3 py-1.5 bg-transparent'),
                h.Placeholder('Suchen …'),
                h.Value(model.search),
                h.OnInput(query => TypedSearch({ query })),
              ]),
            ],
          ),
        ],
      ),
      model.error ? h.div([h.Class('text-red-600 mb-4')], [model.error]) : h.empty,
      h.div(
        [h.Class('rounded-[10px] border border-black/10 dark:border-white/10 overflow-hidden')],
        [
          h.table(
            [h.Class('w-full')],
            [
              h.thead(
                [h.Class('text-left text-sm text-black/40 dark:text-white/40 border-b border-black/10 dark:border-white/10')],
                [
                  h.tr(
                    [],
                    [
                      h.th([h.Class('py-2 px-3 font-normal')], ['Buchung']),
                      h.th([h.Class('py-2 font-normal')], ['Konto']),
                      h.th([h.Class('py-2 font-normal')], ['Kategorie']),
                      h.th([h.Class('py-2 px-3 font-normal text-right')], ['Betrag']),
                    ],
                  ),
                ],
              ),
              h.tbody(
                [h.Class('px-3')],
                groupByBookingDate(model.transactions).flatMap(([date, transactions]) => [
                  h.tr([], [h.td([h.Class('pt-4 pb-1 px-3 text-sm text-black/40 dark:text-white/40'), h.Attribute('colspan', '4')], [formatBookingDate(date)])]),
                  ...transactions.flatMap(transaction => transactionRow(h, transaction)),
                ]),
              ),
            ],
          ),
        ],
      ),
    ],
  )

const contractRow = (h: ReturnType<typeof html<Message>>, contract: Contract) =>
  h.tr(
    [],
    [
      h.td([h.Class('py-1.5')], [contract.normalized_counterparty]),
      h.td([h.Class('py-1.5 text-sm text-black/40 dark:text-white/40')], [contract.interval]),
      h.td([h.Class('py-1.5 text-right tabular-nums')], [formatAmountCents(contract.expected_amount_cents)]),
    ],
  )

const kategorienScreen = (h: ReturnType<typeof html<Message>>, model: Model) => {
  const parentCategories = model.categories.filter(c => c.parent_id === null)

  return h.div(
    [h.Class('flex-1 p-8 overflow-y-auto')],
    [
      h.div([h.Class('text-2xl font-semibold mb-6')], ['Kategorien']),
      model.categoriesError ? h.div([h.Class('text-red-600 mb-4')], [model.categoriesError]) : h.empty,
      h.div(
        [h.Class('flex gap-6')],
        [
          h.div(
            [h.Class('w-64 shrink-0 flex flex-col gap-1')],
            parentCategories.map(category =>
              h.keyed('div')(
                category.id,
                [
                  h.Class(
                    category.id === model.selectedCategoryId
                      ? 'px-3 py-2 rounded-[10px] bg-black/5 dark:bg-white/10 font-medium cursor-pointer'
                      : 'px-3 py-2 rounded-[10px] cursor-pointer hover:bg-black/5 dark:hover:bg-white/5',
                  ),
                  h.OnClick(SelectedCategory({ categoryId: category.id })),
                ],
                [category.name],
              ),
            ),
          ),
          h.div(
            [h.Class('flex-1 rounded-[10px] border border-black/10 dark:border-white/10 p-6')],
            model.categoryDetailError
              ? [h.div([h.Class('text-red-600')], [model.categoryDetailError])]
              : !model.categoryDetail
                ? [h.div([h.Class('text-black/40 dark:text-white/40')], ['Lädt …'])]
                : [
                    h.div(
                      [h.Class('flex items-baseline justify-between mb-4')],
                      [
                        h.div([h.Class('text-xl font-semibold')], [model.categoryDetail.name]),
                        h.div([h.Class('text-lg tabular-nums')], [formatAmountCents(-model.categoryDetail.spent_cents)]),
                      ],
                    ),
                    model.categoryDetail.subcategories.length > 0
                      ? h.table(
                          [h.Class('w-full mb-6')],
                          [
                            h.tbody(
                              [],
                              model.categoryDetail.subcategories.map(sub =>
                                h.tr(
                                  [],
                                  [
                                    h.td([h.Class('py-1')], [sub.name]),
                                    h.td([h.Class('py-1 text-right tabular-nums')], [formatAmountCents(-sub.spent_cents)]),
                                  ],
                                ),
                              ),
                            ),
                          ],
                        )
                      : h.empty,
                    model.categoryDetail.contracts.length > 0
                      ? h.div(
                          [],
                          [
                            h.div([h.Class('text-sm text-black/40 dark:text-white/40 mb-2')], ['Verträge']),
                            h.table([h.Class('w-full')], [h.tbody([], model.categoryDetail.contracts.map(c => contractRow(h, c)))]),
                          ],
                        )
                      : h.empty,
                  ],
          ),
        ],
      ),
    ],
  )
}

const budgetStateLabel = (state: string): string =>
  state === 'over' ? '⚠ überschritten' : state === 'warning' ? '▲ 80 % erreicht' : 'im Rahmen'

const budgetRow = (h: ReturnType<typeof html<Message>>, row: BudgetRow) => {
  const percent = row.target_cents > 0 ? Math.min((row.spent_cents / row.target_cents) * 100, 100) : 0
  const barColor = row.state === 'over' ? 'bg-red-500' : row.state === 'warning' ? 'bg-amber-500' : 'bg-emerald-500'

  return h.div(
    [h.Class('py-3 border-b border-black/5 dark:border-white/5 last:border-0')],
    [
      h.div(
        [h.Class('flex items-baseline justify-between mb-1')],
        [
          h.div([h.Class('font-medium')], [row.category_name]),
          h.div(
            [h.Class('text-sm tabular-nums text-black/60 dark:text-white/60')],
            [`${formatAmountCents(-row.spent_cents).replace('−', '')} / ${formatAmountCents(row.target_cents).replace('+', '')}`],
          ),
        ],
      ),
      h.div(
        [h.Class('h-2 rounded-full bg-black/5 dark:bg-white/10 overflow-hidden')],
        [h.div([h.Class(`h-full rounded-full ${barColor}`), h.Style({ width: `${percent}%` })], [])],
      ),
      h.div([h.Class('text-xs text-black/40 dark:text-white/40 mt-1')], [budgetStateLabel(row.state)]),
    ],
  )
}

const budgetScreen = (h: ReturnType<typeof html<Message>>, model: Model) => {
  const expenseCategories = model.categories.filter(c => c.kind === 'expense')

  return h.div(
    [h.Class('flex-1 p-8 overflow-y-auto')],
    [
      h.div([h.Class('text-2xl font-semibold mb-6')], ['Budget']),
      model.budgetOverviewError ? h.div([h.Class('text-red-600 mb-4')], [model.budgetOverviewError]) : h.empty,
      !model.budgetOverview
        ? h.div([h.Class('text-black/40 dark:text-white/40')], ['Lädt …'])
        : h.div(
            [h.Class('rounded-[10px] border border-black/10 dark:border-white/10 p-6 mb-6')],
            [
              ...model.budgetOverview.rows.map(row => budgetRow(h, row)),
              h.div(
                [h.Class('flex items-baseline justify-between pt-3 text-black/40 dark:text-white/40')],
                [
                  h.div([], ['Ohne Budget']),
                  h.div([h.Class('tabular-nums')], [formatAmountCents(-model.budgetOverview.unbudgeted_expense_cents)]),
                ],
              ),
            ],
          ),
      h.div(
        [h.Class('rounded-[10px] border border-black/10 dark:border-white/10 p-4 flex gap-2 items-center')],
        [
          h.select(
            [
              h.Class('rounded-[10px] border border-black/10 dark:border-white/20 px-3 py-1.5 bg-transparent'),
              h.OnChange(value => TypedBudgetFormCategory({ categoryId: value === '' ? null : Number(value) })),
            ],
            [
              h.option([h.Value('')], ['Kategorie wählen …']),
              ...expenseCategories.map(category =>
                h.option(
                  [h.Value(String(category.id)), h.Selected(model.budgetFormCategoryId === category.id)],
                  [category.name],
                ),
              ),
            ],
          ),
          h.input([
            h.Class('rounded-[10px] border border-black/10 dark:border-white/20 px-3 py-1.5 bg-transparent w-32'),
            h.Placeholder('Betrag €'),
            h.Value(model.budgetFormAmount),
            h.OnInput(amount => TypedBudgetFormAmount({ amount })),
          ]),
          h.button(
            [
              h.Class('rounded-[10px] bg-black text-white dark:bg-white dark:text-black px-4 py-1.5'),
              h.OnClick(ClickedSetBudgetTarget()),
            ],
            ['Setzen'],
          ),
          model.budgetFormError ? h.div([h.Class('text-red-600 text-sm')], [model.budgetFormError]) : h.empty,
        ],
      ),
    ],
  )
}

const screenTitles: Record<Screen, string> = {
  Uebersicht: 'Übersicht',
  Transaktionen: 'Transaktionen',
  Kategorien: 'Kategorien',
  Budget: 'Budget',
}

const screenView = (h: ReturnType<typeof html<Message>>, model: Model) => {
  switch (model.screen) {
    case 'Uebersicht':
      return uebersichtScreen(h, model)
    case 'Kategorien':
      return kategorienScreen(h, model)
    case 'Budget':
      return budgetScreen(h, model)
    case 'Transaktionen':
      return transaktionenScreen(h, model)
  }
}

export const view = (model: Model): Document => {
  const h = html<Message>()

  return {
    title: `mifi — ${screenTitles[model.screen]}`,
    body: h.div(
      [h.Class('min-h-screen flex bg-[#f9f9f7] dark:bg-[#0d0d0d] text-black dark:text-white font-sans')],
      [sidebar(h, model), screenView(h, model)],
    ),
  }
}
