import { View, Text } from 'react-native';

const LabelAndItem = ({ label, children }: { label: string; children: React.ReactNode }) => {
  return (
    <>
      <View className="mb-4 border-2 border-solid border-red-100">
        <Text className="text-2xl font-bold color-black">{label}</Text>
        {children}
      </View>
    </>
  );
};

export default LabelAndItem;
