import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { CourseApp } from './courses/CourseApp';
import './styles.css';
import './lessons/lessons.css';

const root = document.getElementById('root');
if (!root) throw new Error('Application root is missing.');
createRoot(root).render(<StrictMode><CourseApp /></StrictMode>);
