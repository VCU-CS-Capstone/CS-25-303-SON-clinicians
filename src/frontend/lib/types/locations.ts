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
  function fullName(program: Program): string {
    switch (program) {
      case Program.RHWP:
        return 'Richmond Health And Wellness Program';
      case Program.MHWP:
        return 'Mobile Health And Wellness Program';
    }
  }
}
