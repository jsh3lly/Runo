use core::panic;

use crate::card::*;

macro_rules! game_logic_bug_panic {
    () => {
        panic!("Game Logic Error: Please report that this happend")
    };
}

/// Remember: top_card and player_card will have a color at this point.
pub fn verify_move(player_card: Card, top_card: Card, card_debt: usize, ) -> Result<(), String> {
    if card_debt > 0 {
        match (top_card.color, top_card.kind, player_card.color, player_card.kind) {
            (Some(_), CardKind::Draw2, Some(_), CardKind::Draw2 | CardKind::Draw4) => Ok(()),
            (Some(t_c), CardKind::Draw4, Some(p_c), CardKind::Draw2) if p_c == t_c => Ok(()),
            (Some(_), CardKind::Draw4, Some(_), CardKind::Draw4) => Ok(()),
            (None, _, None, _) => game_logic_bug_panic!(),
            _ => Err("Cannot play this card.".to_string()),
        }
    }
    else {
        // match (player_card.kind, top_card.kind) {
        //     (CardKind::Draw4 | CardKind::Wild, _) => Ok(()),
        // }
        match (top_card.color, player_card.color, top_card.kind, player_card.kind, top_card.number, player_card.number) {
            // // If top card is Draw2 or Draw4 but card_debt == 0, then logic_bug!!
            // (_, _, CardKind::Draw2 | CardKind::Draw4, _, _, _) => game_logic_bug_panic!(), 
            // If colors are same, can play any kind
            (Some(t_c), Some(p_c), _, _, _, _) if t_c == p_c => Ok(()), 
            // If kinds are same but not numbers
            (Some(_), Some(_), t_k, p_k, None, None) if t_k == p_k => Ok(()),
            // if player card is Draw2 or Draw4
            (Some(_), Some(_), _, CardKind::Wild | CardKind::Draw4, _, _) => Ok(()),
            // if top card and player card is number kind and numbers are same
            (_, _, CardKind::Number, CardKind::Number, Some(t_n), Some(p_n)) if t_n == p_n => Ok(()),
            _ => Err("Cannot play this card.".to_string()),
        }
    }
    // match player_card.kind {
    //     CardKind::Draw4 => Ok(()),
    //     CardKind::Wild if card_debt != 0 => Ok(()),
    //     _ => {
    //         match top_card.kind {
    //             CardKind::Number if card_debt == 0 => {
    //                 match (top_card.color, top_card.number, player_card.color, player_card.number) {
    //                     (Some(t_c), Some(_), Some(p_c), Some(_)) if t_c == p_c => Ok(()),
    //                     (Some(_), Some(t_n), Some(_), Some(p_n)) if t_n == p_n => Ok(()),
    //                     _ => Err(TurnMoveError::WrongColorOrNumberError)
    //                 }
    //             }
    //             CardKind::Number if card_debt != 0 => Err(TurnMoveError::NumberCardOnCardDebtError),
    //             CardKind::Skip | CardKind::Reverse if card_debt == 0 => {
    //                 match (top_card.color, top_card.kind, player_card.color, player_card.kind) {
    //                     (Some(t_c), _, Some(p_c), _) if t_c == p_c => Ok(()),
    //                     (Some(_), t_k, Some(_), p_k) if t_k == p_k => Ok(()),
    //                     _ => Err(TurnMoveError::WrongColorOrKindError),
    //                 }
    //             }
    //             CardKind::Skip | CardKind::Reverse if card_debt != 0 => Err(TurnMoveError::NumberCardOnCardDebtError),
    //             CardKind::Draw2 => match player_card.kind {
    //                 CardKind::Draw2 | CardKind::Draw4 => Ok(()),
    //                 _ => Err(TurnMoveError::WrongKindError)
    //             }
    //             
    //             CardKind::Draw4 => match (player_card.kind, top_card.color) {
    //                 (CardKind::Draw2, Some(t_c)) if t_c == player_card.color.unwrap() => Ok(()),
    //                 (CardKind::Draw4, Some(_)) => Ok(()),
    //                 _ => Err(TurnMoveError::WrongKindError),
    //             }
    //             CardKind::Wild if card_debt == 0 && top_card.color.unwrap() => Ok(()),
    //             // CardKind::Wild if card_debt != 0 => Err(TurnMoveError::NumberCardOnCardDebtError),
    //             _ => game_logic_error_panic!(),
    //         }
    //     }
    // }
}
