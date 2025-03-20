import { StyleSheet, Text, View } from 'react-native';
import { HamburgerMenu } from '~/components/menus/hamburger';
import { HamburgerOption } from '~/components/menus/hamburger/HamburgerOption';

export const participantOverViewStyles = StyleSheet.create({
  fullWidth: {
    width: '100%',
  },
  participantName: {
    fontSize: 32,
    fontWeight: 'bold',
    textAlign: 'center',
  },
  flexRowWrap: {
    flexDirection: 'row',
    flexWrap: 'wrap',
  },
  boxHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    borderBottomWidth: 1,
    padding: 16,
  },
  boxHeaderText: {
    fontSize: 24,
    fontWeight: 'bold',
  },
  box: {
    flexBasis: '50%',
    borderWidth: 1,
    padding: 16,
  },
  marginBottom: {
    marginBottom: 16,
  },
  yesOrNoText: {
    fontSize: 24,
    fontWeight: 'bold',
  },
  halfWidthBox: {
    width: '50%',
    borderWidth: 1,
    padding: 16,
  },
  flexRowBetween: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
});
export function YesOrNo({ label, value }: { label: string; value?: boolean }) {
  return (
    <View style={participantOverViewStyles.marginBottom}>
      <Text style={participantOverViewStyles.yesOrNoText}>
        {label}: {value ? 'Yes' : 'No'}
      </Text>
    </View>
  );
}
export function BoxHeader({ title }: { title: string }) {
  return (
    <View style={participantOverViewStyles.boxHeader}>
      <Text style={participantOverViewStyles.boxHeaderText}>{title}</Text>
      <HamburgerMenu iconWidth={36} iconHeight={36}>
        <HamburgerOption title="Open Page" />
        <HamburgerOption title="Edit Page" />
      </HamburgerMenu>
    </View>
  );
}
