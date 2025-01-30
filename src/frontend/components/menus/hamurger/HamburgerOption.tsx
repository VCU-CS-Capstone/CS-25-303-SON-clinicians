import { Pressable, StyleSheet, Text } from 'react-native';

export interface HamburgerOptionProps {
  title: string;
  onPress?: () => void;
}
export const HamburgerOption = ({ title, onPress }: HamburgerOptionProps) => {
  return (
    <Pressable onPress={onPress} style={{ padding: 8 }}>
      <Text style={styles.text}>{title}</Text>
    </Pressable>
  );
};

const styles = StyleSheet.create({
  text: {
    fontSize: 16,
  },
});
