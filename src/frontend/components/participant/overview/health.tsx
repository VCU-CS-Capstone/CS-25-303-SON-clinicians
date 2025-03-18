import { Text, View } from 'react-native';
import { ParticipantHealthOverview } from '~/lib/types/participant';
import { BoxHeader, YesOrNo } from '.';
import LabelAndItem from '~/components/LabelAndItem';
import { useEffect, useState } from 'react';
import api from '~/lib/api';
import { participantOverViewStyles as styles } from '~/components/participant/overview';

export function HealthOverviewBox({ particpantId }: { particpantId: number }) {
  const [participantHealthOverview, setParticipantHealthOverview] = useState<
    ParticipantHealthOverview | undefined
  >(undefined);
  const [particpantExists, setParticpantExists] = useState<boolean | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const fetchPatient = async () => {
    try {
      const healthOverview = await api.participants.fetchHealthOverview(particpantId);
      if (healthOverview.data) {
        setParticipantHealthOverview(healthOverview.data);
      }
      setParticpantExists(healthOverview.participant_exists);
      setError(undefined);
      setLoading(false);
    } catch (e: any) {
      setError(e.message as string);
      setLoading(false);
    }
  };
  useEffect(() => {
    fetchPatient();
  }, [particpantId]);

  return (
    <View style={styles.box}>
      <BoxHeader title="Health Overview" />
      <LoadedHealthOverviewBox
        healthOverview={participantHealthOverview}
        loading={loading}
        participantExists={particpantExists}
      />
    </View>
  );
}
function LoadedHealthOverviewBox({
  healthOverview,
  loading,
  participantExists,
}: {
  healthOverview: ParticipantHealthOverview | undefined;
  loading: boolean;
  participantExists: boolean | undefined;
}) {
  if (!healthOverview && loading) {
    return (
      <View>
        <Text>Loading...</Text>
      </View>
    );
  } else if (!healthOverview) {
    if (participantExists === false) {
      return (
        <View>
          <Text>Participant does not exist</Text>
        </View>
      );
    }
    return (
      <View>
        <Text>Participant does does not have Health Overview Filled out</Text>
      </View>
    );
  }
  return (
    <View>
      <View>
        <YesOrNo
          label="Takes More Than 5 Medications"
          value={healthOverview.takes_more_than_5_medications}
        />
        <YesOrNo
          label="Has Blood Pressure Cuff"
          value={healthOverview.takes_more_than_5_medications}
        />
      </View>
      <LabelAndItem label="Allergies">
        <Text>{healthOverview.reported_health_conditions || 'None'}</Text>
      </LabelAndItem>
      <LabelAndItem label="Reported Health Conditions">
        <Text>{healthOverview.reported_health_conditions || 'None'}</Text>
      </LabelAndItem>
    </View>
  );
}
