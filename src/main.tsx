import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import { ApplicationProvider } from './application/ApplicationProvider';
import { nativeLessonPort } from './lessons/nativeLessonPort';
import './styles.css';
import './lessons/lessons.css';

const root = document.getElementById('root');
if (!root) throw new Error('Application root is missing.');
createRoot(root).render(<StrictMode><ApplicationProvider ports={{ lessons: nativeLessonPort }}><App /></ApplicationProvider></StrictMode>);
