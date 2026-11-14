# Human verification (only checks the agent cannot perform)

Build and launch from the project folder: `cargo run --release`
The title screen and the journal show the build (e.g. "v1.0.0-rc1 (build <hash>)"). Your save is in the project root and
backed up to `save_backups/` on every launch.

## Controller mapping (from src/pad.rs; labels are Xbox-layout names of the gilrs buttons)
| Pad button (gilrs) | Game action (keyboard equivalent) |
|---|---|
| A (South) | interact / confirm / next (E, Enter) |
| B (East) | cancel / close (Esc) |
| X (West) | use tool / dive (Space) |
| Y (North) | pockets; in journal: mute music (I, X) |
| RB (RightTrigger) | cycle tool / rotate furniture (Tab, R) |
| LB (LeftTrigger) | pick up furniture (G) |
| LT / RT (Trigger2) | run (Shift) |
| View/Back (Select) | move furniture (F) |
| Menu (Start) | journal (J); also claims the pad |
| hold View + press Menu | switch this pad between Player 1 and Player 2 (P2 leaves if no pad remains on P2) |
| D-pad / left stick | move, menu navigation (arrows) |
On PlayStation-style pads the same positions apply (Cross = A, Circle = B, Square = X, Triangle = Y). Not physically verified.

## H1 Keyboard, single player
Walk (WASD/arrows), run (Shift), press E at your door, open pockets (I), close (Esc), walk out of the room. No stuck keys.

## H2 Two physical gamepads
1. Connect two pads; press Menu (Start) or A on one: the screen splits and that pad is Player 2.
2. Check every action in the mapping table above on both pads, including menu navigation and hold-to-repeat.
3. Player 1 (keyboard or pad) and Player 2 open different menus at the same time; neither input affects the other's menu.
4. With one pad: hold View and press Menu. The pad now controls Player 1 and the screen returns to one view; repeat to go back to Player 2.
5. Unplug Player 2's pad: "Controller disconnected"; plug it back in and press Menu: control returns with nothing lost.

## H3 Release features with real input (keyboard and each pad)
- Title screen: Continue, Controls and guide, Quit (Quit saves and closes).
- Journal (J / Menu): Tab / RB cycles Collection, Initiatives, Settings, Controls, Guide; settings change music/SFX volume.
- Campground (east meadow): talk to today's visitor, buy an item, try the favor, read the sign; check the other player sees "sold out".
- Town Hall: Hazel's desk (policies, funded project as Player 1), the initiatives kiosk (claim a finished task, trade coupons).
- Housing: expand at the shop counter, pay part of the balance, confirm the next expansion is blocked until paid.
- Gullrock: travel (fare or Ferry Pass), Pell's task, the three timed activities alone and together.

## H4 Listening
Music plays and changes with the hour/weather/indoors; footsteps and effects are audible on your output device (HDMI TV).
The agent confirmed through the Windows audio meter that the game produces sound on the TV endpoint, not that it is heard.

## H5 Playtest judgments
Activity thresholds, initiative quantities, coupon prices, campground prices and the overall pace feel fair and fun.

Record results (pass/fail + notes) below.

## H6 Depth pass (2026-11-14)
- Talk to residents across days: tier lines, seasonal/gossip/hobby lines, best-friend portrait at high friendship; Friends page in the journal.
- Festival days (set the PC date or wait): plaza host, decorations, task and keepsake; festive music arrangement.
- Daily surprise: find it (residents hint east/west), return a lost scarf to its owner.
- General Store renovation letters and the weekly special after enough spending.
- M2: buy seeds/can from the counter menu entry, plant and water over several real days, cross two colours; upgrade a tool and feel the difference; repaint your house; read a Monday Home Society letter.
