import { Story } from 'foldkit'
import { describe, expect, test } from 'vitest'

import { FetchedTransactions, FetchTransactions, SelectedAccount, TypedSearch, type Model, update } from './main'

const initialModel: Model = {
  accounts: [
    { id: 1, name: 'Consorsbank Giro' },
    { id: 2, name: 'Consorsbank Tagesgeld' },
  ],
  transactions: [],
  selectedAccountId: null,
  search: '',
  loading: false,
  error: null,
}

describe('update', () => {
  test('SelectedAccount updates the filter and triggers a refetch', () => {
    Story.story(
      update,
      Story.with(initialModel),
      Story.message(SelectedAccount({ accountId: 1 })),
      Story.Command.expectHas(FetchTransactions),
      Story.model(model => {
        expect(model.selectedAccountId).toBe(1)
        expect(model.loading).toBe(true)
      }),
      Story.Command.resolve(FetchTransactions, FetchedTransactions({ transactions: [] })),
    )
  })

  test('SelectedAccount with null shows all accounts again', () => {
    Story.story(
      update,
      Story.with({ ...initialModel, selectedAccountId: 1 }),
      Story.message(SelectedAccount({ accountId: null })),
      Story.model(model => {
        expect(model.selectedAccountId).toBeNull()
      }),
      Story.Command.resolve(FetchTransactions, FetchedTransactions({ transactions: [] })),
    )
  })

  test('TypedSearch updates the query and triggers a refetch', () => {
    Story.story(
      update,
      Story.with(initialModel),
      Story.message(TypedSearch({ query: 'rewe' })),
      Story.Command.expectHas(FetchTransactions),
      Story.model(model => {
        expect(model.search).toBe('rewe')
      }),
      Story.Command.resolve(FetchTransactions, FetchedTransactions({ transactions: [] })),
    )
  })
})
