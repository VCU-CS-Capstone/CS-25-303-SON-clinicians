import { Text, View } from 'react-native';
import { ParticipantDemographics } from '~/lib/types/participant';
import { BoxHeader, YesOrNo } from '.';
import { participantOverViewStyles as styles } from '~/components/participant/overview';
import LabelAndItem from '~/components/LabelAndItem';
import { ShowInsurances } from '../HealthInsurance';
import { useEffect, useState } from 'react';
import api from '~/lib/api';
export function ParticipantDemographicsBox({ particpantId }: { particpantId: number }) {
  const [participantsDemograhics, setParticipantDemographics] = useState<
    ParticipantDemographics | undefined
  >(undefined);
  const [particpantExists, setParticpantExists] = useState<boolean | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const fetchPatient = async () => {
    try {
      const result = await api.participants.fetchDemographic(particpantId);
      if (result.data) {
        setParticipantDemographics(result.data);
      }
      setParticpantExists(result.participant_exists);
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
      <BoxHeader title="Demographics" />
      <LoadedData
        demographics={participantsDemograhics}
        loading={loading}
        participantExists={particpantExists}
      />
    </View>
  );
}
function LoadedData({
  demographics,
  loading,
  participantExists,
}: {
  demographics: ParticipantDemographics | undefined;
  loading: boolean;
  participantExists: boolean | undefined;
}) {
  if (!demographics && loading) {
    return (
      <View>
        <Text>Loading...</Text>
      </View>
    );
  } else if (!demographics) {
    if (participantExists === false) {
      return (
        <View>
          <Text>Participant does not exist</Text>
        </View>
      );
    }
    return (
      <View>
        <Text>Participant does does not have demographics Filled out</Text>
      </View>
    );
  }
  return (
    <View>
      <View style={styles.flexRowBetween}>
        <LabelAndItem label="Age">
          <Text>{demographics.age}</Text>
        </LabelAndItem>
        <YesOrNo label="Is Veteran" value={demographics.is_veteran} />
      </View>
      <LabelAndItem label="Health Insurace">
        <ShowInsurances insurances={demographics.health_insurance || []} />
      </LabelAndItem>
    </View>
  );
}
