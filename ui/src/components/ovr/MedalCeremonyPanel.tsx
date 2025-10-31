import React, { useMemo, useState } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';

const MedalCeremonyPanel: React.FC = () => {
  const medalistPlaceholders = Array.from({ length: 4 });
  const [calendarDate, setCalendarDate] = useState(() => {
    const now = new Date();
    return { month: now.getMonth(), year: now.getFullYear() };
  });

  const calendarMatrix = useMemo(() => {
    const { month, year } = calendarDate;
    const firstOfMonth = new Date(year, month, 1);
    const daysInMonth = new Date(year, month + 1, 0).getDate();
    const leadingEmpty = (firstOfMonth.getDay() + 6) % 7; // Monday as first day
    const totalCells = Math.ceil((leadingEmpty + daysInMonth) / 7) * 7;
    const matrix: Array<Array<{ label: number; inMonth: boolean }>> = [];

    for (let index = 0; index < totalCells; index += 7) {
      const week: Array<{ label: number; inMonth: boolean }> = [];
      for (let cell = 0; cell < 7; cell++) {
        const dayIndex = index + cell - leadingEmpty + 1;
        week.push({
          label: dayIndex,
          inMonth: dayIndex >= 1 && dayIndex <= daysInMonth,
        });
      }
      matrix.push(week);
    }

    return matrix;
  }, [calendarDate]);

  const goToPreviousMonth = () => {
    setCalendarDate(({ month, year }) => {
      if (month === 0) {
        return { month: 11, year: year - 1 };
      }
      return { month: month - 1, year };
    });
  };

  const goToNextMonth = () => {
    setCalendarDate(({ month, year }) => {
      if (month === 11) {
        return { month: 0, year: year + 1 };
      }
      return { month: month + 1, year };
    });
  };

  const monthYearLabel = useMemo(() => {
    const formatter = new Intl.DateTimeFormat(undefined, {
      month: 'long',
      year: 'numeric',
    });
    return formatter.format(new Date(calendarDate.year, calendarDate.month, 1));
  }, [calendarDate]);

  return (
    <div className="grid gap-6 xl:grid-cols-[280px,minmax(0,1fr)] xl:auto-rows-max">
      <section className="theme-card flex h-full flex-col space-y-4 p-4 xl:row-span-2">
        <header className="border-b border-gray-800 pb-3">
          <h2 className="text-lg font-semibold text-gray-100">Ceremony playlist</h2>
        </header>

        <div className="flex-1 space-y-4">
          <div className="rounded-md border border-gray-800 bg-gray-950/50">
            <div className="border-b border-gray-800 px-3 py-2 text-sm font-medium text-gray-200">
              Division title
            </div>
            <div className="px-3 py-3 space-y-2 text-sm text-gray-300">
              <div>G · Athlete · IOC · Flag</div>
              <div>S · Athlete · IOC · Flag</div>
              <div>B · Athlete · IOC · Flag</div>
              <div>B · Athlete · IOC · Flag</div>
            </div>
          </div>

          <div className="flex items-center justify-between gap-3">
            <Label className="text-sm text-gray-300">Position</Label>
            <Input className="w-24" type="number" min={1} defaultValue={1} />
          </div>
        </div>
      </section>

      <section className="theme-card space-y-6 p-4 xl:col-start-2 xl:row-start-1">
        <header className="border-b border-gray-800 pb-3">
          <h2 className="text-lg font-semibold text-gray-100">Settings</h2>
        </header>

        <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,200px)_minmax(0,1fr)]">
          <div className="space-y-4">
            <div className="flex flex-col gap-2">
              <Label className="text-sm text-gray-300">Background image</Label>
              <div className="flex h-28 items-center justify-center rounded-lg border border-dashed border-gray-700 bg-gray-900/60 text-center text-xs text-gray-400">
                Double-click to select background image
              </div>
            </div>

            <div className="flex flex-col gap-2">
              <Label className="text-sm text-gray-300">Break image</Label>
              <div className="flex h-28 items-center justify-center rounded-lg border border-dashed border-gray-700 bg-gray-900/60 text-center text-xs text-gray-400">
                Double-click to select break image
              </div>
            </div>
          </div>

          <div className="space-y-4">
            <div className="space-y-2">
              <Label className="text-sm text-gray-300">Duration</Label>
              <Input type="number" min={0} defaultValue={12} className="w-full" />
            </div>
            <div className="space-y-2">
              <Label className="text-sm text-gray-300">Transition</Label>
              <Input type="number" min={0} defaultValue={12} className="w-full" />
            </div>
            <div className="grid gap-2">
              <Button variant="primary">Play/Pause</Button>
              <Button variant="secondary">Stop</Button>
              <Button variant="secondary">Next</Button>
              <Button variant="secondary">Reset</Button>
              <Button variant="secondary">Show/Hide</Button>
            </div>
          </div>

          <div className="space-y-4">
            <div className="rounded-lg border border-gray-800 bg-gray-950/40 p-4 text-sm text-gray-300">
              <div className="mb-3 flex items-center justify-between">
                <button
                  type="button"
                  onClick={goToPreviousMonth}
                  className="rounded-md border border-gray-700 px-2 py-1 text-xs text-gray-200 transition hover:bg-gray-800"
                  aria-label="Previous month"
                >
                  ‹
                </button>
                <div className="font-medium text-gray-100">{monthYearLabel}</div>
                <button
                  type="button"
                  onClick={goToNextMonth}
                  className="rounded-md border border-gray-700 px-2 py-1 text-xs text-gray-200 transition hover:bg-gray-800"
                  aria-label="Next month"
                >
                  ›
                </button>
              </div>

              <div className="grid grid-cols-7 gap-1 text-center text-[11px] uppercase tracking-wide text-gray-400">
                {['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'].map((day) => (
                  <div key={day} className="py-1">
                    {day}
                  </div>
                ))}
              </div>

              <div className="mt-1 grid grid-cols-7 gap-1 text-center text-sm">
                {calendarMatrix.map((week, weekIndex) =>
                  week.map((day, dayIndex) => (
                    <div
                      key={`${weekIndex}-${dayIndex}`}
                      className={`rounded-md py-2 ${
                        day.inMonth
                          ? 'bg-gray-900/70 text-gray-100'
                          : 'bg-gray-900/30 text-gray-600'
                      }`}
                    >
                      {day.label}
                    </div>
                  )),
                )}
              </div>
            </div>

            <div className="flex flex-wrap items-center gap-2">
              <Button variant="secondary">New</Button>
              <Button variant="primary">Save</Button>
              <Button variant="danger">Delete</Button>
            </div>
          </div>
        </div>
      </section>

      <section className="theme-card space-y-5 p-4 xl:col-start-2 xl:row-start-2">
        <header className="border-b border-gray-800 pb-3">
          <h2 className="text-lg font-semibold text-gray-100">Ceremony playlist items</h2>
        </header>

        <div className="grid gap-4 md:grid-cols-[minmax(0,1fr)_minmax(0,220px)]">
          <div className="space-y-3">
            <div className="space-y-2">
              <Label className="text-sm text-gray-300">Tournament / Day</Label>
              <Input placeholder="Tournament name · Day" />
            </div>
            <div className="space-y-2">
              <Label className="text-sm text-gray-300">Division</Label>
              <Input placeholder="Division name" />
            </div>
          </div>
          <div className="flex h-full items-end">
            <Button variant="secondary" className="w-full">
              Populate automatically from tournament
            </Button>
          </div>
        </div>

        <div className="space-y-3 rounded-lg border border-gray-800 bg-gray-950/40 p-4">
          {medalistPlaceholders.map((_, index) => (
            <div
              key={index}
              className="grid gap-3 sm:grid-cols-[minmax(0,1fr)_160px_140px]"
            >
              <Input placeholder="Medalist name (free input)" />
              <Button variant="secondary" className="w-full">
                IOC animation
              </Button>
              <Button variant="secondary" className="w-full">
                Anthem
              </Button>
            </div>
          ))}
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <Button variant="secondary">New</Button>
          <Button variant="primary">Save</Button>
          <Button variant="danger">Delete</Button>
        </div>
      </section>
    </div>
  );
};

export default MedalCeremonyPanel;
