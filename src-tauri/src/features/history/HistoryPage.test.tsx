import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { HistoryPage } from './HistoryPage';

describe('HistoryPage', () => {
  it('should render history page', () => {
    render(<HistoryPage />);
    
    expect(screen.getByText('历史记录')).toBeInTheDocument();
  });

  it('should show empty state', () => {
    render(<HistoryPage />);
    
    expect(screen.getByText('暂无记录')).toBeInTheDocument();
  });
});
