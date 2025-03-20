import { View, Text } from 'react-native';
import { StyleSheet } from 'react-native';

const LabelAndItem = ({ label, children }: { label: string; children: React.ReactNode }) => {
  return (
    <View style={styles.container}>
      <View style={styles.labelContainer}>
        <Text style={styles.label}>{label}</Text>
      </View>
      <View style={styles.contentContainer}>{children}</View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    marginBottom: 16,
    borderBottomWidth: 2,
    borderBottomColor: '#FFCDD2',
  },
  label: {
    fontSize: 24,
    color: 'black',
    fontWeight: 'bold',
    padding: 12,
  },
  labelContainer: {},
  contentContainer: {
    marginLeft: 20,
  },
});
export default LabelAndItem;
