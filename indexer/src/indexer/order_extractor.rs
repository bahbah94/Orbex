use anyhow::Result;
use subxt::ext::scale_value::{Composite, Primitive, Value};

// ============================================
// Data structures for events
// ============================================

#[derive(Debug, Clone)]
pub struct OrderPlacedData {
    pub order_id: u128,
    pub side: String, // "Buy" or "Sell"
    pub asset_id: u32,
    pub price: u128,
    pub quantity: u128,
}

#[derive(Debug, Clone)]
pub struct OrderFilledData {
    pub order_id: u128,
    pub trader: String,
}

#[derive(Debug, Clone)]
pub struct OrderPartiallyFilledData {
    pub order_id: u128,
    pub trader: String,
    pub filled_quantity: u128,
    pub remaining_quantity: u128,
}

#[derive(Debug, Clone)]
pub struct OrderCancelledData {
    pub order_id: u128,
    pub trader: String,
}

// ============================================
// Extract OrderPlaced event
// ============================================

pub fn extract_order_placed(composite: &Composite<u32>) -> Result<OrderPlacedData> {
    if let Composite::Named(fields) = composite {
        let mut order_id = 0u128;
        let mut side = String::new();
        let mut asset_id = 0u32;
        let mut price = 0u128;
        let mut quantity = 0u128;

        for (name, field_value) in fields {
            match name.as_str() {
                "order_id" => {
                    order_id = extract_u128_from_value(field_value)?;
                }
                "side" => {
                    // side is a variant: Buy() or Sell()
                    side = extract_side_variant(field_value)?;
                }
                "asset_id" => {
                    asset_id = extract_u32_from_value(field_value)?;
                }
                "price" => {
                    price = extract_u128_from_value(field_value)?;
                }
                "quantity" => {
                    quantity = extract_u128_from_value(field_value)?;
                }
                _ => {}
            }
        }

        Ok(OrderPlacedData {
            order_id,
            side,
            asset_id,
            price,
            quantity,
        })
    } else {
        Err(anyhow::anyhow!("OrderPlaced: Not a named composite"))
    }
}

// ============================================
// Extract OrderFilled event
// ============================================

pub fn extract_order_filled(composite: &Composite<u32>) -> Result<OrderFilledData> {
    if let Composite::Named(fields) = composite {
        let mut order_id = 0u128;
        let mut trader = String::new();

        for (name, field_value) in fields {
            match name.as_str() {
                "order_id" => {
                    order_id = extract_u128_from_value(field_value)?;
                }
                "trader" => {
                    trader = extract_account_from_value(field_value)?;
                }
                _ => {}
            }
        }

        Ok(OrderFilledData { order_id, trader })
    } else {
        Err(anyhow::anyhow!("OrderFilled: Not a named composite"))
    }
}

// ============================================
// Extract OrderPartiallyFilled event
// ============================================

pub fn extract_order_partially_filled(
    composite: &Composite<u32>,
) -> Result<OrderPartiallyFilledData> {
    if let Composite::Named(fields) = composite {
        let mut order_id = 0u128;
        let mut trader = String::new();
        let mut filled_quantity = 0u128;
        let mut remaining_quantity = 0u128;

        for (name, field_value) in fields {
            match name.as_str() {
                "order_id" => {
                    order_id = extract_u128_from_value(field_value)?;
                }
                "trader" => {
                    trader = extract_account_from_value(field_value)?;
                }
                "filled_quantity" => {
                    filled_quantity = extract_u128_from_value(field_value)?;
                }
                "remaining_quantity" => {
                    remaining_quantity = extract_u128_from_value(field_value)?;
                }
                _ => {}
            }
        }

        Ok(OrderPartiallyFilledData {
            order_id,
            trader,
            filled_quantity,
            remaining_quantity,
        })
    } else {
        Err(anyhow::anyhow!(
            "OrderPartiallyFilled: Not a named composite"
        ))
    }
}

// ============================================
// Extract OrderCancelled event
// ============================================

pub fn extract_order_cancelled(composite: &Composite<u32>) -> Result<OrderCancelledData> {
    if let Composite::Named(fields) = composite {
        let mut order_id = 0u128;
        let mut trader = String::new();

        for (name, field_value) in fields {
            match name.as_str() {
                "order_id" => {
                    order_id = extract_u128_from_value(field_value)?;
                }
                "trader" => {
                    trader = extract_account_from_value(field_value)?;
                }
                _ => {}
            }
        }

        Ok(OrderCancelledData { order_id, trader })
    } else {
        Err(anyhow::anyhow!("OrderCancelled: Not a named composite"))
    }
}

// ============================================
// Helper extraction functions
// ============================================

/// Extract u128 from a Value
fn extract_u128_from_value(value: &Value<u32>) -> Result<u128> {
    if let subxt::ext::scale_value::ValueDef::Primitive(Primitive::U128(val)) = &value.value {
        Ok(*val)
    } else {
        Err(anyhow::anyhow!(
            "Expected u128 primitive, got {:?}",
            value.value
        ))
    }
}

/// Extract u32 from a Value (stored as u128 in subxt)
fn extract_u32_from_value(value: &Value<u32>) -> Result<u32> {
    if let subxt::ext::scale_value::ValueDef::Primitive(Primitive::U128(val)) = &value.value {
        Ok(*val as u32)
    } else {
        Err(anyhow::anyhow!(
            "Expected u128 primitive (for u32), got {:?}",
            value.value
        ))
    }
}

/// Extract side variant (Buy or Sell)
/// The side is stored as: Variant { name: "Buy" or "Sell", values: ... }
fn extract_side_variant(value: &Value<u32>) -> Result<String> {
    if let subxt::ext::scale_value::ValueDef::Variant(variant) = &value.value {
        Ok(variant.name.clone())
    } else {
        Err(anyhow::anyhow!(
            "Expected Variant for side, got {:?}",
            value.value
        ))
    }
}

/// Extract account address (32 bytes as hex string)
/// Structure:
/// Composite(Unnamed([
///     Composite(Unnamed([U128, U128, ..., U128])) <- 32 bytes
/// ]))
fn extract_account_from_value(value: &Value<u32>) -> Result<String> {
    if let subxt::ext::scale_value::ValueDef::Composite(Composite::Unnamed(outer)) = &value.value {
        if let Some(first) = outer.first() {
            if let subxt::ext::scale_value::ValueDef::Composite(Composite::Unnamed(bytes)) =
                &first.value
            {
                let mut account_bytes = Vec::new();
                for byte_val in bytes {
                    if let subxt::ext::scale_value::ValueDef::Primitive(Primitive::U128(b)) =
                        &byte_val.value
                    {
                        account_bytes.push(*b as u8);
                    }
                }
                if account_bytes.len() == 32 {
                    return Ok(format!("0x{}", hex::encode(&account_bytes)));
                }
            }
        }
    }
    Err(anyhow::anyhow!("Failed to extract account address"))
}
