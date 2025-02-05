import { useState } from 'react';
import { Pressable, StyleSheet, View } from 'react-native';
import Animated, { useAnimatedStyle, withTiming } from 'react-native-reanimated';
import { Line, Path, Svg } from 'react-native-svg';

export interface IconProps {
  isOpen: boolean;
  onPress: () => void;
  width?: number;
  height?: number;
}
const AnimatedLine = Animated.createAnimatedComponent(Line);

export const HamburgerIcon = ({ isOpen, onPress, width, height }: IconProps) => {
  const actualWidth = width || 24;
  const actualHeight = height || 24;
  const topLineStyle = useAnimatedStyle(() => ({
    transform: [
      { rotate: withTiming(isOpen ? '45deg' : '0deg') },
      { translateY: withTiming(isOpen ? -6 : 0) },
      { translateX: withTiming(isOpen ? 6 : 0) },
    ],
  }));

  const middleLineStyle = useAnimatedStyle(() => ({
    opacity: withTiming(isOpen ? 0 : 1),
  }));

  const bottomLineStyle = useAnimatedStyle(() => ({
    transform: [
      { rotate: withTiming(isOpen ? '-45deg' : '0deg') },
      { translateY: withTiming(isOpen ? 0 : 0) },
      { translateX: withTiming(isOpen ? -12 : 0) },
    ],
  }));

  return (
    <Pressable onPress={onPress}>
      <View style={styles.container}>
        <Svg height={actualHeight} width={actualWidth} style={styles.svgStyle} viewBox="0 0 24 24">
          <AnimatedLine
            x1="4"
            y1="6"
            x2="20"
            y2="6"
            stroke="black"
            strokeWidth="2"
            animatedProps={topLineStyle}
          />
          <AnimatedLine
            x1="4"
            y1="12"
            x2="20"
            y2="12"
            stroke="black"
            strokeWidth="2"
            animatedProps={middleLineStyle}
          />
          <AnimatedLine
            x1="4"
            y1="18"
            x2="20"
            y2="18"
            stroke="black"
            strokeWidth="2"
            animatedProps={bottomLineStyle}
          />
        </Svg>
      </View>
    </Pressable>
  );
};
const styles = StyleSheet.create({
  svgStyle: {
    borderColor: 'black',
    borderWidth: 2,
  },
  container: {
    width: 30,
    height: 30,
    justifyContent: 'center',
    alignItems: 'center',
  },
});
