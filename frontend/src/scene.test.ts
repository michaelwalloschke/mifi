import { Scene } from 'foldkit'
import { describe, test } from 'vitest'

import { type Model, update, view } from './main'

const baseModel: Model = {
  screen: 'Transaktionen',
  accounts: [{ id: 1, name: 'Consorsbank Giro' }],
  transactions: [],
  selectedAccountId: null,
  search: '',
  loading: false,
  error: null,
  overview: null,
  overviewError: null,
}

describe('Transaktionen screen', () => {
  test('renders a simple categorized transaction', () => {
    const model: Model = {
      ...baseModel,
      transactions: [
        {
          id: 1,
          booking_date: '2024-05-21',
          account_name: 'Giro',
          counterparty: 'REWE Markt',
          purpose: 'REWE SAGT DANKE 4402',
          amount_cents: -8430,
          is_transfer: false,
          splits: [{ category_name: 'Lebensmittel', amount_cents: -8430, category_source: 'auto' }],
        },
      ],
    }

    Scene.scene(
      { update, view },
      Scene.with(model),
      Scene.expect(Scene.text('REWE Markt')).toExist(),
      Scene.expect(Scene.text('Lebensmittel')).toExist(),
      Scene.expect(Scene.text('−84,30 €')).toExist(),
    )
  })

  test('renders split transactions with indented category rows', () => {
    const model: Model = {
      ...baseModel,
      transactions: [
        {
          id: 1,
          booking_date: '2024-05-21',
          account_name: 'Giro',
          counterparty: 'REWE Markt',
          purpose: 'REWE SAGT DANKE 4402',
          amount_cents: -8430,
          is_transfer: false,
          splits: [
            { category_name: 'Lebensmittel', amount_cents: -6430, category_source: 'auto' },
            { category_name: 'Gesundheit', amount_cents: -2000, category_source: 'user' },
          ],
        },
      ],
    }

    Scene.scene(
      { update, view },
      Scene.with(model),
      Scene.expect(Scene.text('Split (2)')).toExist(),
      Scene.expect(Scene.text('Lebensmittel')).toExist(),
      Scene.expect(Scene.text('Gesundheit')).toExist(),
      Scene.expect(Scene.text('manuell')).toExist(),
    )
  })

  test('renders Transfer legs as excluded from Auswertungen', () => {
    const model: Model = {
      ...baseModel,
      transactions: [
        {
          id: 1,
          booking_date: '2024-05-20',
          account_name: 'Giro',
          counterparty: 'Übertrag Tagesgeld',
          purpose: 'Dauerauftrag Sparen',
          amount_cents: -40000,
          is_transfer: true,
          splits: [{ category_name: null, amount_cents: -40000, category_source: 'auto' }],
        },
      ],
    }

    Scene.scene(
      { update, view },
      Scene.with(model),
      Scene.expect(Scene.text('⇄ Umbuchung — nicht in Auswertungen')).toExist(),
    )
  })
})

describe('Übersicht screen', () => {
  const overviewModel: Model = {
    ...baseModel,
    screen: 'Uebersicht',
    overview: {
      current: { month: '2024-05', einnahmen_cents: 442500, ausgaben_cents: 293900, sparquote_percent: 33.6, puffer_cents: 148600 },
      previous: { month: '2024-04', einnahmen_cents: 420000, ausgaben_cents: 288700, sparquote_percent: 31.3, puffer_cents: 131300 },
      sparkline: [
        { month: '2024-04', einnahmen_cents: 420000, ausgaben_cents: 288700, sparquote_percent: 31.3, puffer_cents: 131300 },
        { month: '2024-05', einnahmen_cents: 442500, ausgaben_cents: 293900, sparquote_percent: 33.6, puffer_cents: 148600 },
      ],
    },
  }

  test('renders the four stat tiles with real figures', () => {
    Scene.scene(
      { update, view },
      Scene.with(overviewModel),
      Scene.expect(Scene.text('Einnahmen')).toExist(),
      Scene.expect(Scene.text('+4.425,00 €')).toExist(),
      Scene.expect(Scene.text('Ausgaben')).toExist(),
      Scene.expect(Scene.text('Sparquote')).toExist(),
      Scene.expect(Scene.text('34 %')).toExist(),
      Scene.expect(Scene.text('Puffer übrig')).toExist(),
    )
  })

  test('shows a loading state before the overview arrives', () => {
    Scene.scene(
      { update, view },
      Scene.with({ ...baseModel, screen: 'Uebersicht', overview: null }),
      Scene.expect(Scene.text('Lädt …')).toExist(),
    )
  })
})
