// Automated End-to-End Verification Test for Car Shopper MCP Server
import assert from 'node:assert';
import { McpProtocolServer } from './mcp_server.js';
import { relayManager } from './relay.js';

async function runTests() {
  console.log('🧪 Starting Car Shopper MCP End-to-End Verification...\n');
  const server = new McpProtocolServer();

  // Test 1: MCP initialize
  console.log('1. Testing MCP "initialize"...');
  const initResp = await server.handleMessage({
    jsonrpc: '2.0',
    id: 1,
    method: 'initialize',
    params: {
      protocolVersion: '2024-11-05',
      capabilities: {},
      clientInfo: { name: 'test-client', version: '1.0.0' }
    }
  });
  assert.strictEqual(initResp.id, 1);
  assert.strictEqual(initResp.result.serverInfo.name, 'car-shopper-mcp');
  console.log('   ✓ Protocol initialized successfully.');

  // Test 2: MCP tools/list
  console.log('2. Testing MCP "tools/list"...');
  const listResp = await server.handleMessage({
    jsonrpc: '2.0',
    id: 2,
    method: 'tools/list',
    params: {}
  });
  assert.strictEqual(listResp.id, 2);
  const toolNames = listResp.result.tools.map(t => t.name);
  console.log(`   ✓ Found ${toolNames.length} tools: ${toolNames.join(', ')}`);
  assert(toolNames.includes('search_cars'));
  assert(toolNames.includes('create_anonymous_alias'));
  assert(toolNames.includes('send_dealer_inquiry'));
  assert(toolNames.includes('check_inbox'));
  assert(toolNames.includes('reveal_contact_info'));

  // Test 3: Search cars (e.g. Honda Accord)
  console.log('3. Testing tool "search_cars" (Honda)...');
  const searchResp = await server.handleMessage({
    jsonrpc: '2.0',
    id: 3,
    method: 'tools/call',
    params: {
      name: 'search_cars',
      arguments: { make: 'Honda' }
    }
  });
  assert.strictEqual(searchResp.id, 3);
  assert.strictEqual(searchResp.result.isError, false);
  const searchData = JSON.parse(searchResp.result.content[0].text);
  assert(searchData.count >= 1);
  const targetCar = searchData.results[0];
  console.log(`   ✓ Search returned: ${targetCar.title} (${targetCar.price}) at ${targetCar.dealer}`);

  // Test 4: Get Car Details
  console.log('4. Testing tool "get_car_details"...');
  const detailResp = await server.handleMessage({
    jsonrpc: '2.0',
    id: 4,
    method: 'tools/call',
    params: {
      name: 'get_car_details',
      arguments: { car_id: targetCar.car_id }
    }
  });
  const detailData = JSON.parse(detailResp.result.content[0].text);
  assert(detailData.vin);
  assert.strictEqual(detailData.dealer.id, 'dealer-boston-honda');
  console.log(`   ✓ Verified details for VIN: ${detailData.vin}, Dealer: ${detailData.dealer.name}`);

  // Test 5: Create Anonymous Persona / Alias
  console.log('5. Testing tool "create_anonymous_alias"...');
  const aliasResp = await server.handleMessage({
    jsonrpc: '2.0',
    id: 5,
    method: 'tools/call',
    params: {
      name: 'create_anonymous_alias',
      arguments: {
        car_id: targetCar.car_id,
        buyer_display_name: 'Alex'
      }
    }
  });
  const aliasData = JSON.parse(aliasResp.result.content[0].text);
  assert(aliasData.alias_id);
  assert(aliasData.anonymous_email.includes('@shopper-relay.net'));
  const aliasId = aliasData.alias_id;
  console.log(`   ✓ Provisioned anonymous alias: ${aliasData.anonymous_email} (ID: ${aliasId})`);

  // Test 6: Send Dealer Inquiry
  console.log('6. Testing tool "send_dealer_inquiry"...');
  const inquiryResp = await server.handleMessage({
    jsonrpc: '2.0',
    id: 6,
    method: 'tools/call',
    params: {
      name: 'send_dealer_inquiry',
      arguments: {
        alias_id: aliasId,
        inquiry_type: 'out_the_door_price',
        subject: 'Out-The-Door pricing breakdown request',
        message_body: 'Hello, what is the complete out-the-door price including doc fees and sales tax? Is this vehicle still on the lot?'
      }
    }
  });
  const inquiryData = JSON.parse(inquiryResp.result.content[0].text);
  assert.strictEqual(inquiryData.status, 'sent');
  console.log(`   ✓ Outbound inquiry sent to dealership: "${inquiryData.subject}"`);

  // Verify inquiry appeared in dealer's inbox
  const dealerInquiries = relayManager.getDealerInbox('dealer-boston-honda');
  assert(dealerInquiries.length > 0);
  console.log(`   ✓ Dealership CRM received message from: ${dealerInquiries[0].from}`);

  // Test 7: Dealer responds with an itemized quote
  console.log('7. Simulating Dealer response...');
  relayManager.receiveDealerReply(
    aliasId,
    'Metro Boston Honda Sales',
    'Re: Out-The-Door pricing breakdown request',
    'Good afternoon! Yes, the car is still available. Here is our itemized breakdown:\n' +
    'Selling Price: $24,500\n' +
    'Documentation Fee: $495\n' +
    'State Tax & Reg: $1,690\n' +
    'Total Out-the-Door: $26,685.\n' +
    'Would you like to come in this Saturday for a test drive?'
  );
  console.log('   ✓ Dealer reply delivered to relay.');

  // Test 8: AI assistant checks inbox
  console.log('8. Testing tool "check_inbox"...');
  const inboxResp = await server.handleMessage({
    jsonrpc: '2.0',
    id: 7,
    method: 'tools/call',
    params: {
      name: 'check_inbox',
      arguments: { alias_id: aliasId }
    }
  });
  const inboxData = JSON.parse(inboxResp.result.content[0].text);
  assert.strictEqual(inboxData.message_count, 1);
  const dealerMsg = inboxData.messages[0];
  console.log(`   ✓ Retrieved dealer reply: "${dealerMsg.subject}"`);
  console.log(`   ✓ Price figures detected:`, dealerMsg.key_findings.detectedFigures);
  console.log(`   ✓ Analysis notes:`, dealerMsg.key_findings.notes);
  assert(dealerMsg.key_findings.detectedFigures.includes('$24,500'));
  assert(dealerMsg.key_findings.hasTestDriveInvite === true);

  // Test 9: Buyer is satisfied, reveals real contact info for test drive
  console.log('9. Testing tool "reveal_contact_info"...');
  const revealResp = await server.handleMessage({
    jsonrpc: '2.0',
    id: 8,
    method: 'tools/call',
    params: {
      name: 'reveal_contact_info',
      arguments: {
        alias_id: aliasId,
        full_name: 'Alex Shopper',
        phone_number: '(555) 234-5678',
        note: 'Saturday at 11:00 AM preferred'
      }
    }
  });
  const revealData = JSON.parse(revealResp.result.content[0].text);
  assert.strictEqual(revealData.status, 'contact_revealed');
  console.log(`   ✓ Contact info securely shared with dealership: Name & Phone unlocked.`);

  // Test 10: Check Thread History
  console.log('10. Testing tool "get_conversation_thread"...');
  const threadResp = await server.handleMessage({
    jsonrpc: '2.0',
    id: 9,
    method: 'tools/call',
    params: {
      name: 'get_conversation_thread',
      arguments: { alias_id: aliasId }
    }
  });
  const threadData = JSON.parse(threadResp.result.content[0].text);
  assert.strictEqual(threadData.total_messages, 3); // initial inquiry, dealer reply, contact reveal
  console.log(`   ✓ Complete 3-message audit trail preserved in relay.`);

  console.log('\n🎉 ALL 10 TESTS PASSED SUCCESSFULLY! The Car Shopper MCP POC is fully functional.\n');
}

runTests().catch(err => {
  console.error('❌ Test failed:', err);
  process.exit(1);
});
