import { Story } from 'foldkit'
import { describe, expect, test } from 'vitest'

import {
  ClickedNavItem,
  ClickedSetBudgetTarget,
  FetchedCategories,
  FetchedCategoryDetail,
  FetchedTransactions,
  FetchCategoryDetail,
  FetchTransactions,
  SelectedAccount,
  SelectedCategory,
  SetBudgetTarget,
  SetBudgetTargetFailed,
  TypedBudgetFormAmount,
  TypedBudgetFormCategory,
  TypedSearch,
  type Model,
  update,
} from './main'

const initialModel: Model = {
  screen: 'Transaktionen',
  accounts: [
    { id: 1, name: 'Consorsbank Giro' },
    { id: 2, name: 'Consorsbank Tagesgeld' },
  ],
  transactions: [],
  selectedAccountId: null,
  search: '',
  loading: false,
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

  test('ClickedNavItem switches the active screen without refetching', () => {
    Story.story(
      update,
      Story.with(initialModel),
      Story.message(ClickedNavItem({ screen: 'Uebersicht' })),
      Story.Command.expectNone(),
      Story.model(model => {
        expect(model.screen).toBe('Uebersicht')
      }),
    )
  })

  test('FetchedCategories auto-selects the first parent category', () => {
    Story.story(
      update,
      Story.with(initialModel),
      Story.message(
        FetchedCategories({
          categories: [
            { id: 1, parent_id: null, name: 'Essen & Trinken', kind: 'expense' },
            { id: 2, parent_id: 1, name: 'Supermarkt', kind: 'expense' },
          ],
        }),
      ),
      Story.Command.expectHas(FetchCategoryDetail),
      Story.model(model => {
        expect(model.selectedCategoryId).toBe(1)
      }),
      Story.Command.resolve(
        FetchCategoryDetail,
        FetchedCategoryDetail({
          detail: { id: 1, name: 'Essen & Trinken', kind: 'expense', month: '2024-05', spent_cents: 0, subcategories: [], contracts: [] },
        }),
      ),
    )
  })

  test('SelectedCategory refetches detail for the clicked category', () => {
    Story.story(
      update,
      Story.with(initialModel),
      Story.message(SelectedCategory({ categoryId: 42 })),
      Story.Command.expectHas(FetchCategoryDetail),
      Story.model(model => {
        expect(model.selectedCategoryId).toBe(42)
      }),
      Story.Command.resolve(
        FetchCategoryDetail,
        FetchedCategoryDetail({
          detail: { id: 42, name: 'X', kind: 'expense', month: '2024-05', spent_cents: 0, subcategories: [], contracts: [] },
        }),
      ),
    )
  })

  test('ClickedSetBudgetTarget rejects an unparseable amount without dispatching a Command', () => {
    Story.story(
      update,
      Story.with({ ...initialModel, budgetFormCategoryId: 1, budgetFormAmount: 'abc' }),
      Story.message(ClickedSetBudgetTarget()),
      Story.Command.expectNone(),
      Story.model(model => {
        expect(model.budgetFormError).not.toBeNull()
      }),
    )
  })

  test('ClickedSetBudgetTarget parses German decimal comma into cents', () => {
    Story.story(
      update,
      Story.with({ ...initialModel, budgetFormCategoryId: 1, budgetFormAmount: '400,50' }),
      Story.message(ClickedSetBudgetTarget()),
      Story.Command.expectExact(SetBudgetTarget({ categoryId: 1, amountCents: 40050 })),
      Story.Command.resolve(SetBudgetTarget, SetBudgetTargetFailed({ error: 'ignored in this test' })),
    )
  })

  test('TypedBudgetFormCategory and TypedBudgetFormAmount update the form state', () => {
    Story.story(
      update,
      Story.with(initialModel),
      Story.message(TypedBudgetFormCategory({ categoryId: 7 })),
      Story.message(TypedBudgetFormAmount({ amount: '250' })),
      Story.model(model => {
        expect(model.budgetFormCategoryId).toBe(7)
        expect(model.budgetFormAmount).toBe('250')
      }),
    )
  })
})
