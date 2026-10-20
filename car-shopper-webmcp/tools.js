// MCP Tools Definitions & Execution Handlers
import { searchCars, getCarById } from './inventory.js';
import { relayManager } from './relay.js';

export const TOOLS = [
  {
    name: 'search_cars',
    description: 'Search car listings like CarGurus with filters for make, model, price, mileage, year, and deal rating (Great Deal, Good Deal).',
    inputSchema: {
      type: 'object',
      properties: {
        make: { type: 'string', description: 'Car manufacturer (e.g. Honda, Toyota, Tesla, Ford, Subaru)' },
        model: { type: 'string', description: 'Model name (e.g. Accord, RAV4, Model 3, F-150)' },
        min_price: { type: 'number', description: 'Minimum price in USD' },
        max_price: { type: 'number', description: 'Maximum price in USD' },
        max_mileage: { type: 'number', description: 'Maximum mileage on odometer' },
        min_year: { type: 'number', description: 'Minimum model year (e.g. 2021)' },
        deal_rating: { type: 'string', description: 'Deal rating filter: "great", "good", or "fair"' },
        body_style: { type: 'string', description: 'Body style (Sedan, SUV, Pickup Truck, Wagon)' },
        sort: { 
          type: 'string', 
          enum: ['deal', 'price_asc', 'price_desc', 'mileage_asc'],
          description: 'Sorting criteria' 
        }
      }
    }
  },
  {
    name: 'get_car_details',
    description: 'Get comprehensive details for a specific car including VIN, features, accident history, dealer reputation, and estimated fees.',
    inputSchema: {
      type: 'object',
      properties: {
        car_id: { type: 'string', description: 'Unique vehicle ID (e.g. "car-honda-accord-2022")' }
      },
      required: ['car_id']
    }
  },
  {
    name: 'create_anonymous_alias',
    description: 'Generates a secure, temporary email alias (e.g. buyer_89a@shopper-relay.net) linked to a specific car and dealership. Dealer will never see your real email or identity.',
    inputSchema: {
      type: 'object',
      properties: {
        car_id: { type: 'string', description: 'The ID of the vehicle you are interested in' },
        buyer_display_name: { type: 'string', description: 'Optional display name shown to dealer (defaults to "Verified Auto Buyer")' }
      },
      required: ['car_id']
    }
  },
  {
    name: 'list_active_aliases',
    description: 'List all currently active anonymous shopping aliases, the car and dealer they are negotiating with, and unread dealer replies.',
    inputSchema: {
      type: 'object',
      properties: {}
    }
  },
  {
    name: 'send_dealer_inquiry',
    description: 'Send an inquiry to the dealer using your anonymous alias. The dealer will see your message sent from the relay address. Always review the message with the user before sending.',
    inputSchema: {
      type: 'object',
      properties: {
        alias_id: { type: 'string', description: 'The anonymous alias ID to send from' },
        inquiry_type: { 
          type: 'string', 
          enum: ['out_the_door_price', 'availability', 'window_sticker', 'custom'],
          description: 'Type of inquiry' 
        },
        subject: { type: 'string', description: 'Email subject line' },
        message_body: { type: 'string', description: 'The text of the inquiry to send to the dealer' }
      },
      required: ['alias_id', 'message_body']
    }
  },
  {
    name: 'check_inbox',
    description: 'Check for incoming dealer emails received by your anonymous alias. Automatically extracts quoted prices, doc fees, and sales tactics.',
    inputSchema: {
      type: 'object',
      properties: {
        alias_id: { type: 'string', description: 'Optional: filter by a specific alias ID. If omitted, checks all active aliases.' },
        unread_only: { type: 'boolean', description: 'If true, only returns unread messages. Defaults to false.' }
      }
    }
  },
  {
    name: 'get_conversation_thread',
    description: 'View the complete back-and-forth email history between your anonymous alias and the dealership.',
    inputSchema: {
      type: 'object',
      properties: {
        alias_id: { type: 'string', description: 'The alias ID' }
      },
      required: ['alias_id']
    }
  },
  {
    name: 'reply_to_dealer',
    description: 'Send a follow-up reply in an existing thread through the anonymous relay.',
    inputSchema: {
      type: 'object',
      properties: {
        alias_id: { type: 'string', description: 'The alias ID' },
        message_body: { type: 'string', description: 'Message body to reply to the dealer' }
      },
      required: ['alias_id', 'message_body']
    }
  },
  {
    name: 'reveal_contact_info',
    description: 'Safely reveals your real name and phone number to the dealer once you are happy with the agreed price, in order to schedule an in-person test drive or finalize purchase.',
    inputSchema: {
      type: 'object',
      properties: {
        alias_id: { type: 'string', description: 'The alias ID' },
        full_name: { type: 'string', description: 'Buyer real full name' },
        phone_number: { type: 'string', description: 'Buyer phone number' },
        note: { type: 'string', description: 'Optional message or preferred test drive appointment time' }
      },
      required: ['alias_id', 'full_name', 'phone_number']
    }
  }
];

export async function executeTool(name, args = {}) {
  switch (name) {
    case 'search_cars': {
      const results = searchCars(args);
      return {
        count: results.length,
        results: results.map(c => ({
          car_id: c.id,
          title: `${c.year} ${c.make} ${c.model} ${c.trim}`,
          price: `$${c.price.toLocaleString()}`,
          deal_rating: c.deal_rating,
          savings_vs_market: c.deal_diff < 0 ? `$${Math.abs(c.deal_diff).toLocaleString()} below market` : 'At market',
          mileage: `${c.mileage.toLocaleString()} mi`,
          dealer: `${c.dealer.name} (${c.dealer.city}, ${c.dealer.state})`,
          dealer_rating: `${c.dealer.rating} ⭐ (${c.dealer.reviews_count} reviews)`,
          days_on_market: `${c.days_on_market} days`
        }))
      };
    }

    case 'get_car_details': {
      const car = getCarById(args.car_id);
      if (!car) {
        throw new Error(`Vehicle not found with ID "${args.car_id}"`);
      }
      return {
        id: car.id,
        summary: `${car.year} ${car.make} ${car.model} ${car.trim}`,
        vin: car.vin,
        price: `$${car.price.toLocaleString()}`,
        market_average: `$${car.market_average.toLocaleString()}`,
        deal_rating: car.deal_rating,
        mileage: `${car.mileage.toLocaleString()} miles`,
        exterior_color: car.exterior_color,
        interior_color: car.interior_color,
        transmission: car.transmission,
        drivetrain: car.drivetrain,
        mpg: car.mpg,
        days_on_market: car.days_on_market,
        history: car.history,
        features: car.features,
        dealer: car.dealer,
        recommended_buyer_actions: [
          'Create an anonymous alias before messaging the dealer to protect your personal contact info.',
          'Request an itemized Out-The-Door (OTD) quote with all dealer doc fees, prep fees, and taxes explicitly listed.',
          'Ask whether the dealer installs mandatory add-on packages (e.g. vin etching, tint, paint protection).'
        ]
      };
    }

    case 'create_anonymous_alias': {
      const alias = relayManager.createAlias(args.car_id, args.buyer_display_name);
      return {
        status: 'alias_active',
        alias_id: alias.aliasId,
        anonymous_email: alias.email,
        assigned_to_vehicle: alias.carSummary,
        dealer_name: alias.dealer.name,
        dealer_email: alias.dealer.email,
        privacy_protection: 'Active. Your real email and identity are completely masked.',
        next_step: `Draft an inquiry using send_dealer_inquiry(alias_id="${alias.aliasId}", ...) to request pricing.`
      };
    }

    case 'list_active_aliases': {
      const aliases = relayManager.getAllAliases();
      return {
        count: aliases.length,
        aliases: aliases.map(a => ({
          alias_id: a.aliasId,
          anonymous_email: a.email,
          vehicle: a.carSummary,
          dealer: a.dealer.name,
          unread_messages: a.unreadCount,
          status: a.status,
          created_at: a.createdAt
        }))
      };
    }

    case 'send_dealer_inquiry': {
      const msg = relayManager.sendInquiry(
        args.alias_id,
        args.subject,
        args.message_body,
        args.inquiry_type
      );
      return {
        status: 'sent',
        message_id: msg.id,
        alias_id: msg.aliasId,
        from_anonymous: msg.from,
        to_dealer: msg.to,
        subject: msg.subject,
        sent_at: msg.sentAt,
        delivery_notice: 'Email delivered to dealership inbox through secure anonymous relay.'
      };
    }

    case 'check_inbox': {
      const messages = relayManager.checkInbox(args.alias_id, args.unread_only);
      return {
        message_count: messages.length,
        messages: messages.map(m => ({
          message_id: m.messageId,
          alias_id: m.aliasId,
          from_dealer: m.fromDealer,
          subject: m.subject,
          received_at: m.receivedAt,
          body_preview: m.body,
          key_findings: m.analysis
        }))
      };
    }

    case 'get_conversation_thread': {
      const threadData = relayManager.getThread(args.alias_id);
      return {
        alias_id: threadData.alias.aliasId,
        anonymous_email: threadData.alias.email,
        vehicle: threadData.alias.carSummary,
        dealer: threadData.alias.dealer.name,
        total_messages: threadData.messages.length,
        thread: threadData.messages.map(m => ({
          direction: m.direction,
          from: m.from,
          to: m.to,
          timestamp: m.sentAt || m.receivedAt,
          subject: m.subject,
          body: m.body
        }))
      };
    }

    case 'reply_to_dealer': {
      const alias = relayManager.getAlias(args.alias_id);
      if (!alias) {
        throw new Error(`Alias "${args.alias_id}" not found.`);
      }
      const msg = relayManager.sendInquiry(
        args.alias_id,
        `Re: Inquiry on ${alias.carSummary}`,
        args.message_body,
        'follow_up'
      );
      return {
        status: 'reply_sent',
        message_id: msg.id,
        alias_id: msg.aliasId,
        sent_at: msg.sentAt
      };
    }

    case 'reveal_contact_info': {
      const result = relayManager.revealContactInfo(
        args.alias_id,
        args.full_name,
        args.phone_number,
        args.note
      );
      return result;
    }

    default:
      throw new Error(`Unknown tool "${name}"`);
  }
}
