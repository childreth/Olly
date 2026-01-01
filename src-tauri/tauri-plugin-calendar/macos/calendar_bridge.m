#import <Foundation/Foundation.h>
#import <EventKit/EventKit.h>
#import <objc/message.h>
#import <os/log.h>

// Static singleton store
static EKEventStore *_sharedEventStore = nil;

void reset_calendar_store() {
    _sharedEventStore = nil;
    NSLog(@"[Calendar Plugin] Event store has been reset");
}

EKEventStore *get_calendar_store() {
    static EKAuthorizationStatus lastKnownStatus = -1;
    EKAuthorizationStatus currentStatus = [EKEventStore authorizationStatusForEntityType:EKEntityTypeEvent];
    
    if (_sharedEventStore == nil || (lastKnownStatus != currentStatus && lastKnownStatus != -1)) {
        _sharedEventStore = [[EKEventStore alloc] init];
        lastKnownStatus = currentStatus;
        NSLog(@"[Calendar Plugin] Created fresh EKEventStore instance (Status: %ld)", (long)currentStatus);
    }
    [_sharedEventStore refreshSourcesIfNecessary];
    return _sharedEventStore;
}

// Bridge functions callable from Rust
int calendar_request_permission() {
    __block int result = -1;
    dispatch_semaphore_t semaphore = dispatch_semaphore_create(0);
    
    NSLog(@"[Calendar Plugin] Requesting permission...");

    // UI-triggering permission requests should be initiated from the main thread
    dispatch_async(dispatch_get_main_queue(), ^{
        // Verify Info.plist keys
        NSDictionary *infoDict = [[NSBundle mainBundle] infoDictionary];
        NSString *usageDesc = infoDict[@"NSCalendarsFullAccessUsageDescription"];
        NSString *legacyDesc = infoDict[@"NSCalendarsUsageDescription"];
        
        if (!usageDesc) {
            NSLog(@"[Calendar Plugin] WARNING: NSCalendarsFullAccessUsageDescription is MISSING in Info.plist");
        } else {
            NSLog(@"[Calendar Plugin] Found NSCalendarsFullAccessUsageDescription: %@", usageDesc);
        }
        
        if (!legacyDesc) {
            NSLog(@"[Calendar Plugin] WARNING: NSCalendarsUsageDescription is MISSING in Info.plist");
        }

        EKEventStore *store = get_calendar_store();
        
        // Log current status before requesting
        EKAuthorizationStatus currentStatus = [EKEventStore authorizationStatusForEntityType:EKEntityTypeEvent];
        NSLog(@"[Calendar Plugin] Current Authorization Status: %ld (0=NotDetermined, 1=Restricted, 2=Denied, 3=Authorized, 4=FullAccess, 5=WriteOnly)", (long)currentStatus);

        // Request full access (read and write) for macOS 14+
        SEL fullAccessSel = @selector(requestFullAccessToEventsWithCompletion:);
        
        if ([store respondsToSelector:fullAccessSel]) {
            NSLog(@"[Calendar Plugin] Using macOS 14+ API requestFullAccessToEvents");
            void (*msgSend)(id, SEL, void (^)(BOOL, NSError *)) = (void (*)(id, SEL, void (^)(BOOL, NSError *)))objc_msgSend;
            msgSend(store, fullAccessSel, ^(BOOL granted, NSError *error) {
                NSLog(@"[Calendar Plugin] Full access completion: granted=%d error=%@", granted, error);
                if (granted) {
                    reset_calendar_store(); // Recreate store after permission change
                }
                result = granted ? 1 : 0;
                dispatch_semaphore_signal(semaphore);
            });
        } else {
            NSLog(@"[Calendar Plugin] Using legacy API requestAccessToEntityType");
            #pragma clang diagnostic push
            #pragma clang diagnostic ignored "-Wdeprecated-declarations"
            [store requestAccessToEntityType:EKEntityTypeEvent completion:^(BOOL granted, NSError *error) {
                NSLog(@"[Calendar Plugin] Access completion: granted=%d error=%@", granted, error);
                if (granted) {
                    reset_calendar_store(); // Recreate store after permission change
                }
                result = granted ? 1 : 0;
                dispatch_semaphore_signal(semaphore);
            }];
            #pragma clang diagnostic pop
        }
    });

    // Wait for async operation (timeout after 30 seconds)
    long wait_result = dispatch_semaphore_wait(semaphore, dispatch_time(DISPATCH_TIME_NOW, 30 * NSEC_PER_SEC));
    
    if (wait_result != 0) {
        NSLog(@"[Calendar Plugin] Timeout waiting for permission response");
        return -1;
    }

    return result;
}

int calendar_check_permission() {
    EKAuthorizationStatus status = [EKEventStore authorizationStatusForEntityType:EKEntityTypeEvent];
    if (status == EKAuthorizationStatusNotDetermined) {
        return 0; // Not Determined
    }
    // Status values: 3=Authorized, 4=FullAccess
    if (status == EKAuthorizationStatusAuthorized || status == 4) {
        return 2; // Authorized
    }
    return 1; // Denied / Restricted
}

int calendar_fetch_events(int days_ahead, char **json_ptr) {
    NSLog(@"[Calendar Plugin] Fetching events for next %d days...", days_ahead);

    __block NSString *final_json = nil;
    dispatch_semaphore_t semaphore = dispatch_semaphore_create(0);

    dispatch_async(dispatch_get_main_queue(), ^{
        @try {
            EKEventStore *store = get_calendar_store();

            // Check authorization status
            EKAuthorizationStatus status = [EKEventStore authorizationStatusForEntityType:EKEntityTypeEvent];
            BOOL authorized = (status == EKAuthorizationStatusAuthorized || status == 4);
            
            if (!authorized) {
                NSLog(@"[Calendar Plugin] Not authorized to fetch events. Status: %ld", (long)status);
                final_json = [NSString stringWithFormat:@"{\"events\":[],\"error\":\"Calendar access not authorized (Status: %ld)\"}", (long)status];
                dispatch_semaphore_signal(semaphore);
                return;
            }
            
            // Use local calendar to get events
            NSDate *startDate = [NSDate date];
            NSCalendar *calendar = [NSCalendar currentCalendar];
            NSDate *endDate = [calendar dateByAddingUnit:NSCalendarUnitDay
                                                    value:days_ahead
                                                   toDate:startDate
                                                  options:0];

            // Fetch events
            NSPredicate *predicate = [store predicateForEventsWithStartDate:startDate
                                                                    endDate:endDate
                                                                  calendars:nil];
            
            NSArray<EKEvent *> *events = [store eventsMatchingPredicate:predicate];
            NSLog(@"[Calendar Plugin] Found %lu events", (unsigned long)[events count]);

            // Convert events to JSON array
            NSMutableArray *eventArray = [NSMutableArray array];
            NSISO8601DateFormatter *dateFormatter = [[NSISO8601DateFormatter alloc] init];
            
            for (EKEvent *event in events) {
                @try {
                    NSMutableDictionary *eventDict = [NSMutableDictionary dictionary];
                    eventDict[@"title"] = event.title ?: @"";
                    eventDict[@"startDate"] = [dateFormatter stringFromDate:event.startDate] ?: @"";
                    eventDict[@"endDate"] = [dateFormatter stringFromDate:event.endDate] ?: @"";
                    eventDict[@"location"] = event.location ?: [NSNull null];
                    eventDict[@"notes"] = event.notes ?: [NSNull null];
                    eventDict[@"isAllDay"] = @(event.isAllDay);
                    eventDict[@"isRecurring"] = @(event.hasRecurrenceRules);
                    
                    NSString *calTitle = @"Unknown";
                    if (event.calendar && event.calendar.title) {
                        calTitle = event.calendar.title;
                    }
                    eventDict[@"calendarTitle"] = calTitle;
                    
                    [eventArray addObject:eventDict];
                } @catch (NSException *e) {
                    NSLog(@"[Calendar Plugin] Skipping event due to error: %@", e.reason);
                }
            }
            
            NSDictionary *response = @{@"events": eventArray};
            NSData *jsonData = [NSJSONSerialization dataWithJSONObject:response options:0 error:nil];
            final_json = [[NSString alloc] initWithData:jsonData encoding:NSUTF8StringEncoding];
            
        } @catch (NSException *exception) {
            NSLog(@"[Calendar Plugin] EXCEPTION in fetch: %@ - %@", exception.name, exception.reason);
            final_json = @"{\"events\":[]}";
        }
        dispatch_semaphore_signal(semaphore);
    });

    dispatch_semaphore_wait(semaphore, dispatch_time(DISPATCH_TIME_NOW, 10 * NSEC_PER_SEC));
    
    if (final_json) {
        *json_ptr = strdup([final_json UTF8String]);
        return 0;
    }
    return -1;
}

void calendar_free_string(char *ptr) {
    if (ptr != NULL) {
        free(ptr);
    }
}

int calendar_get_diagnostics(char **json_ptr) {
    NSLog(@"[Calendar Plugin] Gathering diagnostics...");
    
    __block NSString *final_json = nil;
    dispatch_semaphore_t semaphore = dispatch_semaphore_create(0);
    
    dispatch_async(dispatch_get_main_queue(), ^{
        @try {
            EKEventStore *store = get_calendar_store();
            EKAuthorizationStatus status = [EKEventStore authorizationStatusForEntityType:EKEntityTypeEvent];
            
            NSMutableArray *calDiags = [NSMutableArray array];
            NSArray *calendars = [store calendarsForEntityType:EKEntityTypeEvent];
            EKCalendar *defaultCal = [store defaultCalendarForNewEvents];
            
            for (EKCalendar *cal in calendars) {
                [calDiags addObject:@{
                    @"title": cal.title ?: @"Untitled",
                    @"typeCode": @(cal.type),
                    @"allowsContentModifications": @(cal.allowsContentModifications),
                    @"sourceTitle": cal.source.title ?: @"Unknown",
                    @"isDefault": @(defaultCal && [cal.calendarIdentifier isEqualToString:defaultCal.calendarIdentifier])
                }];
            }
            
            NSDictionary *resp = @{
                @"authStatus": @(status),
                @"calendars": calDiags,
                @"defaultCalendar": defaultCal.title ?: [NSNull null]
            };
            
            NSData *data = [NSJSONSerialization dataWithJSONObject:resp options:0 error:nil];
            final_json = [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];
            
        } @catch (NSException *e) {
            final_json = [NSString stringWithFormat:@"{\"authStatus\":-1,\"calendars\":[],\"error\":\"%@\"}", e.reason];
        }
        dispatch_semaphore_signal(semaphore);
    });
    
    dispatch_semaphore_wait(semaphore, dispatch_time(DISPATCH_TIME_NOW, 5 * NSEC_PER_SEC));
    if (final_json) {
        *json_ptr = strdup([final_json UTF8String]);
        return 0;
    }
    return -1;
}

// Helper to try saving to default, then fallback to any writable calendar
BOOL attempt_save_event_robustly(EKEventStore *store, EKEvent *event, NSError **outError) {
    // 1. Try default calendar for new events
    EKCalendar *targetCal = [store defaultCalendarForNewEvents];
    
    // Log default calendar status
    if (targetCal) {
        NSLog(@"[Calendar Plugin] Default calendar: '%@' (Writable: %d)", targetCal.title, targetCal.allowsContentModifications);
        // If default exists but is read-only, discard it so we search for a writable one
        if (!targetCal.allowsContentModifications) {
            targetCal = nil;
        }
    }
    
    // If no usable default, search for ANY writable calendar
    if (!targetCal) {
        NSLog(@"[Calendar Plugin] searching for alternative writable calendar...");
        NSArray *calendars = [store calendarsForEntityType:EKEntityTypeEvent];
        
        // Pass 1: Prefer Local, CalDAV (iCloud), Exchange
        for (EKCalendar *cal in calendars) {
            if (cal.allowsContentModifications && 
                (cal.source.sourceType == EKSourceTypeLocal || 
                 cal.source.sourceType == EKSourceTypeCalDAV || 
                 cal.source.sourceType == EKSourceTypeExchange)) {
                targetCal = cal;
                NSLog(@"[Calendar Plugin] Selected high-priority writable calendar: '%@' (Source: %@)", cal.title, cal.source.title);
                break;
            }
        }
        
        // Pass 2: Accept anything writable if Pass 1 failed
        if (!targetCal) {
            for (EKCalendar *cal in calendars) {
                if (cal.allowsContentModifications) {
                    targetCal = cal;
                    NSLog(@"[Calendar Plugin] Selected fallback writable calendar: '%@' (Source: %@)", cal.title, cal.source.title);
                    break;
                }
            }
        }
    }
    
    if (!targetCal) {
        NSLog(@"[Calendar Plugin] CRITICAL: No writable calendar found in %lu candidates.", (unsigned long)[[store calendarsForEntityType:EKEntityTypeEvent] count]);
        if (outError) *outError = [NSError errorWithDomain:@"OllyCalendarError" code:404 userInfo:@{NSLocalizedDescriptionKey: @"No writable calendar found"}];
        return NO;
    }
    
    event.calendar = targetCal;
    if ([store saveEvent:event span:EKSpanThisEvent commit:YES error:outError]) {
        NSLog(@"[Calendar Plugin] Saved to '%@'", targetCal.title);
        return YES;
    }
    
    NSLog(@"[Calendar Plugin] Failed to save to '%@'. Error: %@", targetCal.title, *outError ? [*outError localizedDescription] : @"Unknown");
    
    // 2. Fallback: Try ALL other writable calendars
    NSArray *calendars = [store calendarsForEntityType:EKEntityTypeEvent];
    for (EKCalendar *cal in calendars) {
        // Skip the one we already tried
        if ([cal.calendarIdentifier isEqualToString:targetCal.calendarIdentifier]) continue;
        
        if (cal.allowsContentModifications) {
            NSLog(@"[Calendar Plugin] Fallback retry on: '%@' (Source: %@)", cal.title, cal.source.title);
            event.calendar = cal;
            NSError *fallbackErr = nil;
            if ([store saveEvent:event span:EKSpanThisEvent commit:YES error:&fallbackErr]) {
                NSLog(@"[Calendar Plugin] Success on fallback: %@", cal.title);
                if (outError) *outError = nil; // Clear previous error
                return YES;
            } else {
                 NSLog(@"[Calendar Plugin] Fallback failed on '%@': %@", cal.title, fallbackErr.localizedDescription);
            }
        }
    }
    
    return NO;
}

// Helper to create a dedicated 'Olly' calendar if no writable one exists
EKCalendar* create_dedicated_calendar(EKEventStore *store) {
    // 0. Check for existing 'Olly' calendar first
    for (EKCalendar *cal in [store calendarsForEntityType:EKEntityTypeEvent]) {
        if ([cal.title isEqualToString:@"Olly"] && cal.allowsContentModifications) {
            NSLog(@"[Calendar Plugin] Found existing 'Olly' calendar.");
            return cal;
        }
    }

    NSLog(@"[Calendar Plugin] Attempting to create dedicated 'Olly' calendar...");
    
    // Find a source that allows creating calendars (Local or iCloud)
    EKSource *targetSource = nil;
    for (EKSource *source in store.sources) {
        NSLog(@"[Calendar Plugin] Found Source: %@ (Type: %ld)", source.title, (long)source.sourceType);
        if (source.sourceType == EKSourceTypeLocal || source.sourceType == EKSourceTypeCalDAV || source.sourceType == EKSourceTypeExchange) {
            // Prefer iCloud (CalDAV) if available
            if (source.sourceType == EKSourceTypeCalDAV && [source.title isEqualToString:@"iCloud"]) {
                targetSource = source;
                break;
            }
            // Use Local as next best
            if (source.sourceType == EKSourceTypeLocal) {
                targetSource = source;
            }
            // Fallback to Exchange or generic CalDAV if nothing else yet
            if (!targetSource) targetSource = source;
        }
    }
    
    // Last ditch: just take the first source?
    if (!targetSource && store.sources.count > 0) {
        targetSource = store.sources.firstObject;
        NSLog(@"[Calendar Plugin] No ideal source found, defaulting to first available: %@", targetSource.title);
    }
    
    if (!targetSource) {
        NSLog(@"[Calendar Plugin] No suitable source found to create calendar.");
        return nil;
    }
    
    EKCalendar *newCal = [EKCalendar calendarForEntityType:EKEntityTypeEvent eventStore:store];
    newCal.title = @"Olly";
    newCal.source = targetSource;
    
    NSError *err = nil;
    if ([store saveCalendar:newCal commit:YES error:&err]) {
        NSLog(@"[Calendar Plugin] Created 'Olly' calendar in source '%@'", targetSource.title);
        return newCal;
    } else {
        NSLog(@"[Calendar Plugin] Failed to create 'Olly' calendar: %@", err.localizedDescription);
        return nil;
    }
}

int calendar_create_event(const char *title, 
                          const char *start_date_iso, 
                          const char *end_date_iso,
                          const char *location,
                          const char *notes,
                          int is_all_day,
                          char **result_json_ptr) {
    NSLog(@"[Calendar Plugin] Creating event: %s", title);
    
    __block int final_result = -1;
    __block NSString *final_json = nil;
    
    dispatch_semaphore_t semaphore = dispatch_semaphore_create(0);
    
    dispatch_async(dispatch_get_main_queue(), ^{
        @try {
            // Use the shared singleton store that has been authorized
            EKEventStore *store = get_calendar_store();
            
            EKAuthorizationStatus status = [EKEventStore authorizationStatusForEntityType:EKEntityTypeEvent];
            BOOL canWrite = (status == EKAuthorizationStatusAuthorized || status == 4 || status == 5);
            
            if (!canWrite) {
                NSDictionary *resp = @{@"success": @NO, @"error": [NSString stringWithFormat:@"Calendar write access not granted (Status: %ld). Please check System Settings > Privacy & Security > Calendars.", (long)status]};
                NSData *data = [NSJSONSerialization dataWithJSONObject:resp options:0 error:nil];
                final_json = [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];
                dispatch_semaphore_signal(semaphore);
                return;
            }
            
            EKEvent *event = [EKEvent eventWithEventStore:store];
            event.title = [NSString stringWithUTF8String:title];
            
            NSISO8601DateFormatter *dateFormatter = [[NSISO8601DateFormatter alloc] init];
            dateFormatter.formatOptions = NSISO8601DateFormatWithInternetDateTime | NSISO8601DateFormatWithFractionalSeconds;
            
            NSString *startStr = [NSString stringWithUTF8String:start_date_iso];
            NSString *endStr = [NSString stringWithUTF8String:end_date_iso];
            NSDate *startDate = [dateFormatter dateFromString:startStr];
            NSDate *endDate = [dateFormatter dateFromString:endStr];
            
            if (!startDate || !endDate) {
                NSDateFormatter *fallback = [[NSDateFormatter alloc] init];
                fallback.dateFormat = @"yyyy-MM-dd'T'HH:mm:ssZ";
                if (!startDate) startDate = [fallback dateFromString:startStr];
                if (!endDate) endDate = [fallback dateFromString:endStr];
            }
            
            if (!startDate || !endDate) {
                NSDictionary *resp = @{@"success": @NO, @"error": @"Invalid date format"};
                NSData *data = [NSJSONSerialization dataWithJSONObject:resp options:0 error:nil];
                final_json = [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];
                dispatch_semaphore_signal(semaphore);
                return;
            }
            
            // Explicitly set time zone to local if not present
            if (startDate && !event.timeZone) {
                event.timeZone = [NSTimeZone localTimeZone];
            }
            
            event.startDate = startDate;
            event.endDate = endDate;
            event.allDay = (is_all_day != 0);
            
            if (location && strlen(location) > 0) event.location = [NSString stringWithUTF8String:location];
            if (notes && strlen(notes) > 0) event.notes = [NSString stringWithUTF8String:notes];
            
            NSError *saveError = nil;
            BOOL success = attempt_save_event_robustly(store, event, &saveError);
            
            // If failed to find ANY writable calendar, try creating one
            if (!success && (!saveError || saveError.code == 404)) {
                EKCalendar *ollyCal = create_dedicated_calendar(store);
                if (ollyCal) {
                    event.calendar = ollyCal;
                    NSError *retryErr = nil;
                    if ([store saveEvent:event span:EKSpanThisEvent commit:YES error:&retryErr]) {
                        success = YES;
                        saveError = nil;
                    } else {
                        saveError = retryErr;
                    }
                }
            }
            
            // Handle ACL Error specifically (Code 1) by resetting store and retrying ONCE
            if (!success && saveError.code == 1 && [saveError.domain isEqualToString:@"EKErrorDomain"]) {
                NSLog(@"[Calendar Plugin] ACL Error. Resetting store and retrying...");
                reset_calendar_store();
                store = get_calendar_store();
                
                EKEvent *retryEvent = [EKEvent eventWithEventStore:store];
                retryEvent.title = event.title;
                retryEvent.startDate = event.startDate;
                retryEvent.endDate = event.endDate;
                retryEvent.allDay = event.allDay;
                retryEvent.location = event.location;
                retryEvent.notes = event.notes;
                retryEvent.timeZone = event.timeZone; // Don't forget timezone!
                
                NSError *retryError = nil;
                success = attempt_save_event_robustly(store, retryEvent, &retryError);
                
                if (success) {
                    event = retryEvent;
                    saveError = nil;
                } else {
                    saveError = retryError;
                }
            }
            
            if (success) {
                NSDictionary *resp = @{@"success": @YES, @"eventId": event.eventIdentifier ?: @"", @"calendarTitle": event.calendar.title ?: @""};
                NSData *data = [NSJSONSerialization dataWithJSONObject:resp options:0 error:nil];
                final_json = [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];
                final_result = 0;
            } else {
                NSString *errorMsg = [NSString stringWithFormat:@"Save failed on '%@'. Error: %@ (Code: %ld, Domain: %@)", 
                                      event.calendar.title, 
                                      saveError.localizedDescription, 
                                      (long)saveError.code,
                                      saveError.domain];
                
                NSDictionary *resp = @{@"success": @NO, @"error": errorMsg};
                NSData *data = [NSJSONSerialization dataWithJSONObject:resp options:0 error:nil];
                final_json = [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];
                final_result = -1;
            }
            
        } @catch (NSException *e) {
            final_json = [NSString stringWithFormat:@"{\"success\":false,\"error\":\"Exception: %@\"}", e.reason];
            final_result = -1;
        }
        dispatch_semaphore_signal(semaphore);
    });
    
    dispatch_semaphore_wait(semaphore, dispatch_time(DISPATCH_TIME_NOW, 30 * NSEC_PER_SEC));
    if (final_json) {
        *result_json_ptr = strdup([final_json UTF8String]);
    } else {
        *result_json_ptr = strdup("{\"success\":false,\"error\":\"Timeout waiting for event creation\"}");
    }
    return final_result;
}
