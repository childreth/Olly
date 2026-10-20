// Anonymous Relay & Persona Manager
import crypto from 'node:crypto';
import { getCarById } from './inventory.js';

class RelayManager {
  constructor() {
    this.aliases = new Map(); // alias_id -> alias data
    this.threads = new Map(); // alias_id -> array of messages
    this.dealerInboxes = new Map(); // dealer_id -> array of dealer-facing messages
    this.listeners = new Set();
  }

  // Subscribe to changes (e.g. for real-time web portal updates)
  subscribe(fn) {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }

  notify(event, data) {
    for (const fn of this.listeners) {
      try {
        fn(event, data);
      } catch (err) {
        console.error('Error in listener:', err);
      }
    }
  }

  createAlias(carId, buyerDisplayName = 'Verified Auto Buyer') {
    const car = getCarById(carId);
    if (!car) {
      throw new Error(`Vehicle with ID "${carId}" not found in inventory.`);
    }

    const shortId = crypto.randomBytes(3).toString('hex');
    const aliasId = `alias_${shortId}`;
    const email = `buyer_${shortId}@shopper-relay.net`;

    const aliasRecord = {
      aliasId,
      email,
      buyerDisplayName,
      carId: car.id,
      carSummary: `${car.year} ${car.make} ${car.model} ${car.trim}`,
      carPrice: car.price,
      vin: car.vin,
      dealer: {
        id: car.dealer.id,
        name: car.dealer.name,
        city: car.dealer.city,
        state: car.dealer.state,
        email: car.dealer.email,
        phone: car.dealer.phone
      },
      status: 'anonymous', // 'anonymous' | 'contact_revealed'
      revealedContact: null,
      createdAt: new Date().toISOString(),
      unreadCount: 0
    };

    this.aliases.set(aliasId, aliasRecord);
    this.threads.set(aliasId, []);

    // Ensure dealer inbox array exists
    if (!this.dealerInboxes.has(car.dealer.id)) {
      this.dealerInboxes.set(car.dealer.id, []);
    }

    this.notify('alias_created', aliasRecord);
    return aliasRecord;
  }

  getAlias(aliasId) {
    return this.aliases.get(aliasId) || null;
  }

  getAllAliases() {
    return Array.from(this.aliases.values());
  }

  sendInquiry(aliasId, subject, messageBody, inquiryType = 'out_the_door_price') {
    const alias = this.getAlias(aliasId);
    if (!alias) {
      throw new Error(`Alias "${aliasId}" does not exist. Create an alias first.`);
    }

    const messageId = `msg_out_${Date.now()}_${crypto.randomBytes(2).toString('hex')}`;
    const message = {
      id: messageId,
      aliasId,
      direction: 'outbound',
      from: `${alias.buyerDisplayName} <${alias.email}>`,
      to: `${alias.dealer.name} <${alias.dealer.email}>`,
      subject: subject || `Inquiry on ${alias.carSummary} (VIN: ${alias.vin})`,
      body: messageBody,
      inquiryType,
      sentAt: new Date().toISOString()
    };

    // Store in buyer thread
    const thread = this.threads.get(aliasId) || [];
    thread.push(message);
    this.threads.set(aliasId, thread);

    // Deliver to dealer portal inbox
    const dealerInbox = this.dealerInboxes.get(alias.dealer.id) || [];
    dealerInbox.unshift({
      ...message,
      read: false
    });
    this.dealerInboxes.set(alias.dealer.id, dealerInbox);

    this.notify('inquiry_sent', { alias, message });
    return message;
  }

  // Called when dealer replies (via simulator portal or inbound webhook)
  receiveDealerReply(aliasId, senderName, subject, messageBody) {
    const alias = this.getAlias(aliasId);
    if (!alias) {
      throw new Error(`Alias "${aliasId}" does not exist.`);
    }

    const messageId = `msg_in_${Date.now()}_${crypto.randomBytes(2).toString('hex')}`;
    
    // Parse prices and detect sales tactics
    const parsedData = this.analyzeDealerMessage(messageBody, alias.carPrice);

    const message = {
      id: messageId,
      aliasId,
      direction: 'inbound',
      from: `${senderName || alias.dealer.name} <${alias.dealer.email}>`,
      to: alias.email,
      subject: subject || `Re: Inquiry on ${alias.carSummary}`,
      body: messageBody,
      receivedAt: new Date().toISOString(),
      read: false,
      analysis: parsedData
    };

    const thread = this.threads.get(aliasId) || [];
    thread.push(message);
    this.threads.set(aliasId, thread);

    alias.unreadCount += 1;
    this.notify('dealer_replied', { alias, message });
    return message;
  }

  // AI-friendly heuristic analysis of dealer emails
  analyzeDealerMessage(text, advertisedPrice) {
    const analysis = {
      detectedFigures: [],
      hasTestDriveInvite: false,
      askedForPhoneNumber: false,
      quotedOutTheDoorPrice: null,
      notes: []
    };

    // Search for currency numbers like $24,500 or $495
    const priceMatches = text.match(/\$[\d,]+(\.\d{2})?/g);
    if (priceMatches) {
      analysis.detectedFigures = Array.from(new Set(priceMatches));
    }

    // Check if dealer asks for buyer's phone number or push for in-person visit
    const lower = text.toLowerCase();
    if (lower.includes('phone') || lower.includes('cell') || lower.includes('call you') || lower.includes('reach you at')) {
      analysis.askedForPhoneNumber = true;
      analysis.notes.push('Dealer is requesting a direct phone number.');
    }

    if (lower.includes('test drive') || lower.includes('come in') || lower.includes('stop by') || lower.includes('schedule a visit')) {
      analysis.hasTestDriveInvite = true;
      analysis.notes.push('Dealer invited you for an on-site test drive / visit.');
    }

    if (lower.includes('out the door') || lower.includes('otd') || lower.includes('total price') || lower.includes('bottom line')) {
      analysis.quotedOutTheDoorPrice = analysis.detectedFigures[0] || 'Present in text';
      analysis.notes.push('Dealer provided out-the-door price breakdown.');
    }

    if (lower.includes('fee') || lower.includes('doc fee') || lower.includes('acquisition') || lower.includes('accessories')) {
      analysis.notes.push('Dealer mentioned dealer fees or add-on packages.');
    }

    return analysis;
  }

  checkInbox(aliasId = null, unreadOnly = false) {
    const results = [];

    const checkSingle = (id) => {
      const alias = this.getAlias(id);
      if (!alias) return [];
      const thread = this.threads.get(id) || [];
      const inbound = thread.filter(m => m.direction === 'inbound');
      const filtered = unreadOnly ? inbound.filter(m => !m.read) : inbound;

      // Mark as read after inspection
      filtered.forEach(m => {
        m.read = true;
      });
      alias.unreadCount = thread.filter(m => m.direction === 'inbound' && !m.read).length;

      return filtered.map(m => ({
        messageId: m.id,
        aliasId: id,
        fromDealer: m.from,
        subject: m.subject,
        receivedAt: m.receivedAt,
        body: m.body,
        analysis: m.analysis
      }));
    };

    if (aliasId) {
      return checkSingle(aliasId);
    } else {
      for (const id of this.aliases.keys()) {
        results.push(...checkSingle(id));
      }
      return results;
    }
  }

  getThread(aliasId) {
    const alias = this.getAlias(aliasId);
    if (!alias) {
      throw new Error(`Alias "${aliasId}" not found.`);
    }
    return {
      alias,
      messages: this.threads.get(aliasId) || []
    };
  }

  revealContactInfo(aliasId, fullName, phoneNumber, note = '') {
    const alias = this.getAlias(aliasId);
    if (!alias) {
      throw new Error(`Alias "${aliasId}" not found.`);
    }

    alias.status = 'contact_revealed';
    alias.revealedContact = {
      fullName,
      phoneNumber,
      revealedAt: new Date().toISOString(),
      note
    };

    // Send formal handover message to dealer
    const handoverMessage = {
      id: `msg_reveal_${Date.now()}`,
      aliasId,
      direction: 'outbound',
      from: `${fullName} (Verified Buyer via Shopper Relay) <${alias.email}>`,
      to: `${alias.dealer.name} <${alias.dealer.email}>`,
      subject: `[Verified Contact Shared] Appointment & Test Drive Request for ${alias.carSummary}`,
      body: `Hello ${alias.dealer.name},\n\nThe verified buyer who was inquiring anonymously under alias ${alias.email} is satisfied with the preliminary details and has authorized releasing their direct contact information to finalize an appointment:\n\nName: ${fullName}\nPhone: ${phoneNumber}\nPreferred timing: ${note || 'Ready to schedule test drive'}\nVehicle: ${alias.carSummary} (VIN: ${alias.vin})\n\nPlease reach out directly or reply to confirm an appointment slot.`,
      inquiryType: 'contact_reveal',
      sentAt: new Date().toISOString()
    };

    const thread = this.threads.get(aliasId) || [];
    thread.push(handoverMessage);

    const dealerInbox = this.dealerInboxes.get(alias.dealer.id) || [];
    dealerInbox.unshift({
      ...handoverMessage,
      read: false
    });

    this.notify('contact_revealed', { alias, handoverMessage });
    return {
      status: 'contact_revealed',
      aliasId,
      email: alias.email,
      dealer: alias.dealer.name,
      handoverMessagePreview: handoverMessage.body
    };
  }

  getDealerInbox(dealerId) {
    return this.dealerInboxes.get(dealerId) || [];
  }
}

export const relayManager = new RelayManager();
