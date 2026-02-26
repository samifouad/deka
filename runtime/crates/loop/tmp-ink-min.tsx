import React from 'react';
import { Text } from 'ink';
import { withFullScreen } from './src/ui/fullscreen';

const ink = withFullScreen(<Text>hi</Text>);
await ink.start();
await ink.waitUntilExit();
