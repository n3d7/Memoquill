import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, expect, it, vi } from 'vitest';
import type { NavigationContext } from './application/workspaceTypes';
import { LibraryApp } from './LibraryApp';
const mocks=vi.hoisted(()=>({ library:vi.fn(),searchResources:vi.fn(),saveNavigation:vi.fn(),preferences:{theme:'system',textScale:100,navigation:null as NavigationContext|null} }));
vi.mock('./contentBridge',async original=>({...await original<typeof import('./contentBridge')>(),library:mocks.library,searchResources:mocks.searchResources}));
vi.mock('./useReadingPreferences',()=>({useReadingPreferences:()=>({preferences:mocks.preferences,loaded:true,working:false,retry:false,saveNavigation:mocks.saveNavigation})}));
vi.mock('./ReadingPreferencesPanel',()=>({ReadingPreferencesPanel:()=>null}));
vi.mock('./MaterialPanel',()=>({MaterialPanel:({resourceId,onDirty}:{resourceId:string;onDirty:(v:boolean)=>void})=><section aria-label="Open material">{resourceId}<button onClick={()=>onDirty(true)}>Simulate unsaved material</button></section>}));
const nav:NavigationContext={contractVersion:1,subjectId:'subject',collectionId:null,resourceId:'resource',query:'Quantum',page:0,searchOffset:64,searchActive:true};
beforeEach(()=>{
 vi.clearAllMocks();mocks.preferences.navigation={...nav};mocks.saveNavigation.mockResolvedValue('Reading preferences saved.');
 mocks.library.mockResolvedValue({workspaceId:'workspace',revision:1,subjects:{subject:{id:'subject',title:'Physics',priorKnowledge:null,archived:false}},resources:{resource:{id:'resource',title:'Quantum reference',archived:false}},collections:{},goals:{},placements:{},units:{},paths:{},relations:{},tags:{}});
 mocks.searchResources.mockResolvedValue({revision:1,items:[{resourceId:'resource',title:'Quantum reference',snippet:''}],nextOffset:null});
});
it('restores a scoped material and reruns the remembered query against current canonical data',async()=>{
 render(<LibraryApp/>);await screen.findByRole('button',{name:'Physics'});fireEvent.click(screen.getByRole('button',{name:'Restore remembered workspace context'}));
 expect(await screen.findByRole('region',{name:'Open material'})).toHaveTextContent('resource');
 await waitFor(()=>expect(mocks.searchResources).toHaveBeenCalledWith('Quantum',64));
 expect(screen.getByLabelText('Search titles, tags and text')).toHaveValue('Quantum');
 fireEvent.click(screen.getByRole('button',{name:'Remember workspace context'}));
 await waitFor(()=>expect(mocks.saveNavigation).toHaveBeenCalledWith(nav));
});
it('protects unsaved material and clears only remembered context',async()=>{
 render(<LibraryApp/>);await screen.findByRole('button',{name:'Physics'});fireEvent.click(screen.getByRole('button',{name:'Restore remembered workspace context'}));
 fireEvent.click(await screen.findByRole('button',{name:'Simulate unsaved material'}));
 expect(screen.getByRole('button',{name:'Restore remembered workspace context'})).toBeDisabled();
 fireEvent.click(screen.getByRole('button',{name:'Forget remembered workspace context'}));
 await waitFor(()=>expect(mocks.saveNavigation).toHaveBeenCalledWith(null));expect(screen.getByRole('region',{name:'Open material'})).toBeInTheDocument();
});
it('leaves foreign or removed selections closed',async()=>{
 mocks.preferences.navigation={...nav,subjectId:'foreign',resourceId:'foreign',searchActive:false};render(<LibraryApp/>);await screen.findByRole('button',{name:'Physics'});fireEvent.click(screen.getByRole('button',{name:'Restore remembered workspace context'}));
 expect(screen.queryByRole('region',{name:'Open material'})).not.toBeInTheDocument();
 expect(screen.getByRole('button',{name:'Physics'})).not.toHaveAttribute('aria-current','page');
});
