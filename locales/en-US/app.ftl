app-title = Turn Checker Companion
app-server-starting = Starting the local sync server...
app-theme-toggle-tooltip = Toggle light/dark mode
app-always-on-top-disable-tooltip = Disable always-on-top
app-always-on-top-enable-tooltip = Keep the app above other windows
app-minimal-mode-disable-tooltip = Return to the full view
app-minimal-mode-enable-tooltip = Switch to a compact overlay

startup-waiting = Waiting...
startup-failed = Startup failed.
startup-unsent-data-title = Unsent data found
startup-unsent-data-message =
    { $count ->
        [one] The current database contains { $count } unsent record.
       *[other] The current database contains { $count } unsent records.
    }
startup-keep-db-question = Do you want to keep the current database?
startup-keep-db-button = Keep current database
startup-reset-db-button = Delete and recreate database

pairing-title = Scan To Connect
pairing-description = Open the iOS app and scan the QR code to configure the server address.
pairing-qr-failed = Failed to generate pairing QR code.
pairing-server-url = Server URL
pairing-server-connection-missing = Server connection info is not available.

action-cancel = Cancel
action-save = Save
action-back = Back
action-restart = Restart
action-next-turn = Next turn

dialog-new-turn-title = New turn
dialog-new-turn-pending-message =
    { $count ->
        [one] You still have { $count } mandatory current-turn check to complete.
       *[other] You still have { $count } mandatory current-turn checks to complete.
    }
dialog-new-turn-blocked-message = The next-turn notification cannot be sent until all mandatory current-turn checks are checked.
dialog-new-turn-confirm-message = Sending a new-turn notification will switch this screen to waiting mode until the next turn arrives.
dialog-restart-title = Restart
dialog-restart-unsent-message =
    { $count ->
        [one] The database contains { $count } unsent record.
       *[other] The database contains { $count } unsent records.
    }
dialog-restart-confirm-message = Restarting will delete and recreate the database, then return to the pairing screen.

content-error-no-current-session = No current session is available yet.
content-new-check-button = New Check
content-source-game-turns-button = Game's turns checks
content-source-game-button = Game's checks
content-source-template-button = Template's checks
content-comments-button = Comments
content-missing-comment-slot = Missing { $comment_type } comment slot.

waiting-next-turn-title = Waiting for next turn...
waiting-next-turn-description = The app will unlock automatically when the new turn is received.

comments-title = Comments
comment-type-game = Game
comment-type-turn = Turn
comments-no-slot = No comment slot is available.

checklist-turn-label = Turn { $turn }
checklist-current-turn = Current turn
checklist-empty = No checks yet.

source-checks-empty = No checks found for { $title }.

new-check-title = Create a new check
field-name = Name
field-detail = Detail
field-source = Source
field-tag = Tag
field-repeat = Repeat
field-repeat-value = Repeat value
field-mandatory = Mandatory
field-no-tag = No tag

source-game = Game
source-global-game = Global Game
source-blueprint = Blueprint
source-turn = Turn

repeat-everytime = Everytime
repeat-conditional = Conditional
repeat-specific = Specific
repeat-until = Until

check-mandatory = Mandatory
repeat-badge-every-turn = Every turn
repeat-badge-conditional = Conditional (Turn { $turn })
repeat-badge-specific = Specific (Turn { $turn })
repeat-badge-until = Until (Turn { $turn })

validation-name-required = Name is required.
validation-field-valid-integer = { $field } must be a valid integer.
validation-field-at-least = { $field } must be at least { $min }.
