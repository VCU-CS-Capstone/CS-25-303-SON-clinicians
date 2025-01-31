export interface Location {
  id: number;
  name: string;
  program: Program;
  parent_location?: number;
}
export enum Program {
  RHWP = 'RHWP',
  MHWP = 'MHWP',
}
export namespace Program {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  export function fullName(program: Program): string {
    switch (program) {
      case Program.RHWP:
        return 'Richmond Health And Wellness Program';
      case Program.MHWP:
        return 'Mobile Health And Wellness Program';
    }
  }
}
export interface LocationWithParentItem {
  id: number;
  name: string;
  program: Program;
  parent_location?: LocationWithParentItem;
}

export function organizeLocationsToWithParents(locations: Location[]): LocationWithParentItem[] {
  const locationsWithParents: LocationWithParentItem[] = [];
  const locationsById: Map<number, LocationWithParentItem> = new Map();
  for (const location of locations) {
    locationsById.set(location.id, {
      id: location.id,
      name: location.name,
      program: location.program,
    });
  }
  for (const location of locations) {
    if (location.parent_location) {
      locationsWithParents.push({
        id: location.id,
        name: location.name,
        program: location.program,
        parent_location: locationsById.get(location.parent_location),
      });
    } else {
      locationsWithParents.push({
        id: location.id,
        name: location.name,
        program: location.program,
      });
    }
  }
  return locationsWithParents;
}
