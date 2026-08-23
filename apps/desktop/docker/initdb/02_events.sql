create table events (
    id            bigint generated always as identity primary key,
    user_id       bigint references users(id) on delete set null,
    event_type    text not null,
    payload       jsonb not null,
    created_at    timestamptz not null default now()
);

create index idx_events_user_id on events(user_id);
create index idx_events_payload_gin on events using gin (payload);

insert into events (user_id, event_type, payload) values
(
    1,
    'checkout.completed',
    '{
        "session": {
            "id": "sess_9f8a3c2e1b7d4f56",
            "startedAt": "2026-08-14T09:12:03Z",
            "completedAt": "2026-08-14T09:19:47Z",
            "durationMs": 464000,
            "device": {
                "type": "desktop",
                "os": "macOS",
                "osVersion": "15.1",
                "browser": "Chrome",
                "browserVersion": "129.0.0.0",
                "screen": { "width": 2560, "height": 1440, "pixelRatio": 2 },
                "locale": "en-US",
                "timezone": "America/Los_Angeles"
            },
            "referrer": "https://www.google.com/search?q=wireless+mouse",
            "utm": {
                "source": "google",
                "medium": "cpc",
                "campaign": "q3-electronics-push",
                "term": "wireless mouse",
                "content": "ad_variant_b"
            }
        },
        "customer": {
            "id": 1,
            "email": "alice@example.com",
            "isReturning": true,
            "lifetimeOrders": 7,
            "lifetimeValueCents": 84920,
            "loyaltyTier": "gold",
            "address": {
                "line1": "482 Market Street",
                "line2": "Suite 300",
                "city": "San Francisco",
                "state": "CA",
                "postalCode": "94105",
                "country": "US"
            }
        },
        "cart": {
            "currency": "USD",
            "items": [
                {
                    "sku": "ELEC-001",
                    "name": "Wireless Mouse",
                    "quantity": 1,
                    "unitPriceCents": 2499,
                    "category": "Electronics",
                    "attributes": { "color": "graphite", "warranty": "2-year" },
                    "discounts": [
                        { "code": "Q3PUSH10", "type": "percentage", "value": 10, "amountCents": 250 }
                    ]
                },
                {
                    "sku": "ELEC-002",
                    "name": "Mechanical Keyboard",
                    "quantity": 1,
                    "unitPriceCents": 8999,
                    "category": "Electronics",
                    "attributes": { "switchType": "brown", "layout": "ANSI", "backlight": true },
                    "discounts": []
                }
            ],
            "subtotalCents": 11498,
            "discountTotalCents": 250,
            "taxCents": 891,
            "shippingCents": 0,
            "totalCents": 12139
        },
        "payment": {
            "method": "card",
            "brand": "visa",
            "last4": "4242",
            "billingAddressMatchesShipping": true,
            "riskScore": 0.02,
            "threeDSecure": { "attempted": true, "authenticated": true }
        },
        "fulfillment": {
            "method": "standard_shipping",
            "estimatedDeliveryDate": "2026-08-19",
            "carrier": "UPS",
            "warehouse": "SFO1"
        },
        "experiments": [
            { "key": "checkout-one-page", "variant": "treatment" },
            { "key": "upsell-widget-v3", "variant": "control" }
        ],
        "flags": {
            "isFraudSuspected": false,
            "isFirstPurchase": false,
            "isGift": false
        }
    }'::jsonb
),
(
    2,
    'page.viewed',
    '{
        "page": {
            "url": "/products/mechanical-keyboard",
            "title": "Mechanical Keyboard — Electronics",
            "path": "/products/mechanical-keyboard",
            "search": "?variant=brown-switch"
        },
        "session": {
            "id": "sess_1a2b3c4d5e6f7089",
            "device": { "type": "mobile", "os": "iOS", "osVersion": "18.0", "browser": "Safari" },
            "referrer": null
        },
        "customer": { "id": 2, "email": "bob@example.com", "isReturning": false },
        "performance": {
            "ttfbMs": 82,
            "domContentLoadedMs": 340,
            "loadEventMs": 611,
            "largestContentfulPaintMs": 590
        },
        "abTests": [{ "key": "product-gallery-zoom", "variant": "treatment" }]
    }'::jsonb
),
(
    null,
    'webhook.delivery_failed',
    '{
        "webhookId": "wh_5c1d9e2a",
        "endpoint": "https://partner.example.com/hooks/orders",
        "attempt": 3,
        "maxAttempts": 5,
        "statusCode": 503,
        "responseBody": "Service Temporarily Unavailable",
        "responseHeaders": {
            "content-type": "text/plain",
            "retry-after": "120"
        },
        "requestPayload": {
            "event": "order.created",
            "orderId": 4,
            "occurredAt": "2026-08-13T22:04:11Z"
        },
        "nextRetryAt": "2026-08-13T22:09:11Z",
        "history": [
            { "attempt": 1, "statusCode": 503, "at": "2026-08-13T21:54:11Z" },
            { "attempt": 2, "statusCode": 503, "at": "2026-08-13T21:59:11Z" },
            { "attempt": 3, "statusCode": 503, "at": "2026-08-13T22:04:11Z" }
        ]
    }'::jsonb
);
