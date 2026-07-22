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

export const Screen = S.Literals(['Uebersicht', 'Transaktionen'])
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
  },
  [FetchAccounts(), FetchTransactions({ accountId: null, search: '' }), FetchOverview()],
]

// VIEW HELPERS

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
  { label: 'Kategorien', screen: null },
  { label: 'Budget', screen: null },
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

export const view = (model: Model): Document => {
  const h = html<Message>()

  return {
    title: `mifi — ${model.screen === 'Uebersicht' ? 'Übersicht' : 'Transaktionen'}`,
    body: h.div(
      [h.Class('min-h-screen flex bg-[#f9f9f7] dark:bg-[#0d0d0d] text-black dark:text-white font-sans')],
      [sidebar(h, model), model.screen === 'Uebersicht' ? uebersichtScreen(h, model) : transaktionenScreen(h, model)],
    ),
  }
}
