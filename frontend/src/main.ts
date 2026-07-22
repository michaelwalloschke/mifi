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

export const Model = S.Struct({
  accounts: S.Array(Account),
  transactions: S.Array(Transaction),
  selectedAccountId: S.NullOr(S.Number),
  search: S.String,
  loading: S.Boolean,
  error: S.NullOr(S.String),
})
export type Model = typeof Model.Type

// MESSAGE

export const FetchedAccounts = m('FetchedAccounts', { accounts: S.Array(Account) })
export const FailedFetchAccounts = m('FailedFetchAccounts', { error: S.String })
export const FetchedTransactions = m('FetchedTransactions', { transactions: S.Array(Transaction) })
export const FailedFetchTransactions = m('FailedFetchTransactions', { error: S.String })
export const SelectedAccount = m('SelectedAccount', { accountId: S.NullOr(S.Number) })
export const TypedSearch = m('TypedSearch', { query: S.String })

export const Message = S.Union([
  FetchedAccounts,
  FailedFetchAccounts,
  FetchedTransactions,
  FailedFetchTransactions,
  SelectedAccount,
  TypedSearch,
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
    }),
  )

// INIT

export const init: Runtime.ApplicationInit<Model, Message> = () => [
  {
    accounts: [],
    transactions: [],
    selectedAccountId: null,
    search: '',
    loading: true,
    error: null,
  },
  [FetchAccounts(), FetchTransactions({ accountId: null, search: '' })],
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

const NAV_ITEMS = ['Übersicht', 'Transaktionen', 'Kategorien', 'Budget', 'Verträge', 'Vermögen', 'Konten'] as const

const sidebar = (h: ReturnType<typeof html<Message>>) =>
  h.div(
    [h.Class('w-[216px] shrink-0 flex flex-col justify-between border-r border-black/10 dark:border-white/10 p-4')],
    [
      h.div(
        [],
        [
          h.div([h.Class('text-lg font-semibold px-2 mb-6')], ['mifi']),
          h.nav(
            [h.Class('flex flex-col gap-1')],
            NAV_ITEMS.map(item =>
              h.div(
                [
                  h.Class(
                    item === 'Transaktionen'
                      ? 'px-2 py-1.5 rounded-[10px] bg-black/5 dark:bg-white/10 font-medium'
                      : 'px-2 py-1.5 rounded-[10px] text-black/40 dark:text-white/40',
                  ),
                ],
                [item],
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
    title: 'mifi — Transaktionen',
    body: h.div(
      [h.Class('min-h-screen flex bg-[#f9f9f7] dark:bg-[#0d0d0d] text-black dark:text-white font-sans')],
      [sidebar(h), transaktionenScreen(h, model)],
    ),
  }
}
