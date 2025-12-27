#import <Foundation/Foundation.h>
#import <EventKit/EventKit.h>
#import <objc/message.h>

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

        __block EKEventStore *store = [[EKEventStore alloc] init];
        
        // Log current status before requesting
        EKAuthorizationStatus currentStatus = [EKEventStore authorizationStatusForEntityType:EKEntityTypeEvent];
        NSLog(@"[Calendar Plugin] Current Authorization Status: %ld (0=NotDetermined, 1=Restricted, 2=Denied, 3=Authorized, 4=FullAccess)", (long)currentStatus);

        SEL fullAccessSel = @selector(requestFullAccessToEventsWithCompletion:);
        if ([store respondsToSelector:fullAccessSel]) {
            NSLog(@"[Calendar Plugin] Using macOS 14+ API requestFullAccessToEvents");
            void (*msgSend)(id, SEL, void (^)(BOOL, NSError *)) = (void (*)(id, SEL, void (^)(BOOL, NSError *)))objc_msgSend;
            msgSend(store, fullAccessSel, ^(BOOL granted, NSError *error) {
                NSLog(@"[Calendar Plugin] Access completion: granted=%d error=%@", granted, error);
                result = granted ? 1 : 0;
                store = nil;
                dispatch_semaphore_signal(semaphore);
            });
        } else {
            NSLog(@"[Calendar Plugin] Using legacy API requestAccessToEntityType");
            #pragma clang diagnostic push
            #pragma clang diagnostic ignored "-Wdeprecated-declarations"
            [store requestAccessToEntityType:EKEntityTypeEvent completion:^(BOOL granted, NSError *error) {
                NSLog(@"[Calendar Plugin] Access completion: granted=%d error=%@", granted, error);
                result = granted ? 1 : 0;
                store = nil;
                dispatch_semaphore_signal(semaphore);
            }];
            #pragma clang diagnostic pop
        }
    });

    // Wait for async operation (timeout after 30 seconds)
    // Since we dispatched to main thread, we must be on a background thread here (which Tauri commands are)
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
    if (status == EKAuthorizationStatusAuthorized || status == 4) {
        return 2; // Authorized
    }
    return 1; // Denied / Restricted
}

int calendar_fetch_events(int days_ahead, char **json_ptr) {
    NSLog(@"[Calendar Plugin] Fetching events for next %d days...", days_ahead);

    @try {
        EKEventStore *store = [[EKEventStore alloc] init];

        // Check authorization status
        EKAuthorizationStatus status;
        BOOL authorized = NO;
        
        status = [EKEventStore authorizationStatusForEntityType:EKEntityTypeEvent];
        if (status == EKAuthorizationStatusAuthorized || status == 4) {
            authorized = YES;
        } else {
            NSLog(@"[Calendar Plugin] Not authorized. Status: %ld", (long)status);
        }
        
        if (!authorized) {
            return -1;
        }
        
        NSLog(@"[Calendar Plugin] Authorization confirmed. Searching events...");

        // Create date range
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
            NSMutableDictionary *eventDict = [NSMutableDictionary dictionary];

            eventDict[@"title"] = event.title ?: @"";
            eventDict[@"startDate"] = [dateFormatter stringFromDate:event.startDate] ?: @"";
            eventDict[@"endDate"] = [dateFormatter stringFromDate:event.endDate] ?: @"";
            
            // Handle nullable properties with NSNull
            eventDict[@"location"] = event.location ?: [NSNull null];
            eventDict[@"notes"] = event.notes ?: [NSNull null];
            eventDict[@"isAllDay"] = @(event.allDay);
            
            if (event.calendar && event.calendar.title) {
                eventDict[@"calendarTitle"] = event.calendar.title;
            } else {
                eventDict[@"calendarTitle"] = [NSNull null];
            }

            [eventArray addObject:eventDict];
        }

        // Wrap in response structure
        NSDictionary *response = @{@"events": eventArray};

        // Convert to JSON string
        NSError *error = nil;
        NSData *jsonData = [NSJSONSerialization dataWithJSONObject:response
                                                           options:0
                                                             error:&error];

        if (error) {
            NSLog(@"[Calendar Plugin] JSON serialization error: %@", error);
            return -1;
        }

        NSString *jsonString = [[NSString alloc] initWithData:jsonData encoding:NSUTF8StringEncoding];
        *json_ptr = strdup([jsonString UTF8String]);
        
        return 0;
    }
    @catch (NSException *exception) {
        NSLog(@"[Calendar Plugin] EXCEPTION: %@ - %@", exception.name, exception.reason);
        return -1;
    }
}

void calendar_free_string(char *ptr) {
    if (ptr != NULL) {
        free(ptr);
    }
}
