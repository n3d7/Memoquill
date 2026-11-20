export interface Course { id: string; title: string }
export interface CourseCatalogue { contractVersion: 1; selectedCourseId: string; courses: Course[] }
export interface CoursePort {
  settings?: import('../settings/types').SettingsPort;
  list(): Promise<CourseCatalogue>;
  create(operationId: string, title: string): Promise<CourseCatalogue>;
  select(courseId: string): Promise<CourseCatalogue>;
}
