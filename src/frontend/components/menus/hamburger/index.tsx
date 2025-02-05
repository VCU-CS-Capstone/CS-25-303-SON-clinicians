import { useState } from 'react';
import { StyleSheet, View } from 'react-native';

import { HamburgerIcon } from './icon';

export interface HamburgerProps {
  children: React.ReactNode;
  iconWidth?: number;
  iconHeight?: number;
}
export const HamburgerMenu = ({ children, iconHeight, iconWidth }: HamburgerProps) => {
  const [isOpen, setIsOpen] = useState(false);
  // TODO: Tap outside to close
  return (
    <View>
      <HamburgerIcon
        width={iconWidth}
        height={iconHeight}
        isOpen={isOpen}
        onPress={() => {
          console.log('pressed');
          setIsOpen(!isOpen);
        }}
      />
      <DropDownMenu isOpen={isOpen}>{children}</DropDownMenu>
    </View>
  );
};

const DropDownMenu = ({ children, isOpen }: { children: React.ReactNode; isOpen: boolean }) => {
  if (!isOpen) {
    return null;
  }
  return <View style={styles.container}>{children}</View>;
};

const styles = StyleSheet.create({
  container: {
    position: 'absolute',
    top: 24,
    right: 0,
    backgroundColor: 'white',
    width: 100,
    zIndex: 1,
  },
  svgStyle: {
    position: 'absolute',
  },
});
