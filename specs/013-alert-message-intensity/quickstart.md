# Quickstart: Intensity Inclusion in Alert Messages

## Overview
This feature ensures that every seismic alert summary includes a clear Japanese description of the intensity (e.g., "震度 3相当の揺れを検出しました").

## Verification

### 1. WebUI History
1. Navigate to the Alert History page in the WebUI.
2. Trigger an alert using the `streamer` tool.
3. Once the alert resets, verify the card displays the message based on the maximum intensity recorded.

### 2. Email Notifications
1. Ensure SMTP is configured and "Email Alerts" are enabled.
2. Trigger an alert.
3. Check the "ALERT SUMMARY" email body. It should contain the descriptive intensity message.

### 3. Log Output
1. Check the `rsudp-rust` console logs.
2. Verify that the `RESET` log includes the descriptive message alongside the raw ratio.
