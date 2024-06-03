module regulated_coin_example::regulated_coin {
    use std::option;

    use sui::coin;
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};

    struct REGULATED_COIN has drop {}

    fun init(otw: REGULATED_COIN, ctx: &mut TxContext) {
        // This creates a new currency, via `create_currency`, but with an extra capability that
        // allows for specific addresses to have their coins frozen. Those addresses cannot interact
        // with the coin as input objects.
        let (treasury_cap, deny_cap, meta_data) = coin::create_regulated_currency(
            otw,
            5,
            b"STABLE$",
            b"RegulaCoin",
            b"Example Regulated Coin",
            option::none(),
            ctx
        );

        let sender = tx_context::sender(ctx);
        transfer::public_transfer(treasury_cap, sender);
        transfer::public_transfer(deny_cap, sender);
        transfer::public_transfer(meta_data, sender);
    }
}
